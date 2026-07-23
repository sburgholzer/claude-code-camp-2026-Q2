from .errors import UnknownToolError
from .tool import Tool


class Registry:
    def __init__(self, context):
        self.context = context

    def tool(self, name, *, description, parameters=None):
        def decorator(block):
            tool = Tool(name, description, parameters or {}, block)
            self.context.register_tool(tool)
            return block

        return decorator

    def dispatch(self, name, args=None):
        tool = self.context.tools.get(name)
        if not tool:
            raise UnknownToolError(f"No tool registered as '{name}'")
        return tool.block(**(args or {}))
