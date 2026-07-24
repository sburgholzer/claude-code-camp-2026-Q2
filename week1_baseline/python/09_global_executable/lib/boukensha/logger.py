import json
import re
import secrets
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_SESSION_DIR = "sessions"


class Logger:
    def __init__(self, *, session_id=None, dir=None, log=None, snapshot=None):
        self.session_id = session_id or self._generate_session_id()
        self.path = log or str(Path(dir or self._default_dir()) / f"{self.session_id}.jsonl")

        Path(self.path).parent.mkdir(parents=True, exist_ok=True)
        self._log_io = open(self.path, "a")
        self._subscribers = []
        self._write_log({"phase": "session_start", **(snapshot or {})})

    def turn(self, *, n):
        self._write_log({"phase": "turn", "n": n})

    def iteration(self, *, n, max):
        self._write_log({"phase": "iteration", "n": n, "max": max})

    def limit_reached(self, *, kind, n, max):
        self._write_log({"phase": "limit_reached", "kind": kind, "n": n, "max": max})

    def turn_end(self, *, reason, iterations, tokens=None):
        self._write_log({"phase": "turn_end", "reason": reason, "iterations": iterations, "tokens": tokens})

    def prompt(self, *, messages, tools):
        self._write_log(
            {
                "phase": "prompt",
                "message_count": len(messages),
                "messages": [self._serialize_message(m) for m in messages],
                "tool_count": len(tools),
                "tools": list(tools.keys()),
            }
        )

    def tool_call(self, *, name, args):
        self._write_log({"phase": "tool_call", "name": name, "args": args})

    def tool_result(self, *, name, result, ok=True, error=None):
        self._write_log({"phase": "tool_result", "name": name, "result": str(result), "ok": ok, "error": error})

    def response(self, *, text, usage=None, stop_reason=None, task=None, backend=None):
        event = {
            "phase": "response",
            "text": str(text).strip(),
            "usage": usage,
            "stop_reason": stop_reason,
        }
        event.update(self._execution_metadata(task=task, backend=backend, usage=usage))
        self._write_log(event)

    def raw(self, *, data):
        from . import is_debug

        if not is_debug():
            return

        self._write_log({"phase": "raw", "data": data})

    def subscribe(self, callback):
        self._subscribers.append(callback)

    def close(self):
        if self._log_io:
            self._log_io.close()

    def _default_dir(self):
        from . import config

        return str(Path(config().dir) / DEFAULT_SESSION_DIR)

    def _write_log(self, event):
        line = {**event, "session_id": self.session_id, "at": datetime.now().astimezone().isoformat()}
        self._log_io.write(json.dumps(line) + "\n")
        self._log_io.flush()
        for subscriber in self._subscribers:
            subscriber(event)

    def _generate_session_id(self):
        timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        return f"{timestamp}-{secrets.token_hex(4)}"

    def _serialize_message(self, msg):
        return {"role": msg.role, "content": msg.content}

    def _execution_metadata(self, *, task, backend, usage):
        if task is None and backend is None and not usage:
            return {}

        tokens = self._usage_tokens(usage)
        metadata = {
            "task": self._task_name(task),
            "provider": self._provider_name(backend),
            "model": backend.model if backend is not None else None,
            "usage_unit": backend.usage_unit if backend is not None and hasattr(backend, "usage_unit") else None,
            "usage_level": backend.usage_level if backend is not None and hasattr(backend, "usage_level") else None,
            "input_tokens": tokens["input"],
            "output_tokens": tokens["output"],
            "cost_usd": self._estimate_cost(backend, tokens),
        }
        return {k: v for k, v in metadata.items() if v is not None}

    def _task_name(self, task):
        if task is not None and hasattr(task, "task_name"):
            return task.task_name()
        return str(task) if task is not None else None

    def _provider_name(self, backend):
        if backend is None:
            return None

        return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", type(backend).__name__).lower()

    def _usage_tokens(self, usage):
        usage = usage or {}
        return {
            "input": self._first_integer(usage, "input_tokens", "prompt_tokens", "promptTokenCount", "prompt_eval_count"),
            "output": self._first_integer(usage, "output_tokens", "completion_tokens", "candidatesTokenCount", "eval_count"),
        }

    def _first_integer(self, data, *keys):
        for key in keys:
            value = data.get(key)
            if value is not None:
                try:
                    return int(value)
                except (TypeError, ValueError):
                    return None
        return None

    def _estimate_cost(self, backend, tokens):
        if backend is None or not hasattr(backend, "estimate_cost"):
            return None
        if tokens["input"] is None or tokens["output"] is None:
            return None

        return backend.estimate_cost(input_tokens=tokens["input"], output_tokens=tokens["output"])
