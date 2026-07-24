from dataclasses import dataclass
from typing import Callable


@dataclass
class Tool:
    name: str
    description: str
    parameters: dict
    block: Callable

    def __repr__(self):
        return (
            f"#<Tool name={self.name} description={self.description[:41]} "
            f"params={list(self.parameters.keys())}>"
        )

    __str__ = __repr__
