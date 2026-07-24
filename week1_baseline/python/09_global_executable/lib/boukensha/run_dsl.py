class RunDSL:
    # Passed to a `boukensha.run` `register` callback. Exposes only `tool`,
    # keeping the DSL surface intentionally small.
    def __init__(self, registry):
        self._registry = registry

    def tool(self, name, *, description, parameters=None):
        return self._registry.tool(name, description=description, parameters=parameters)
