#!/usr/bin/env python3
"""Exercise an example module's dependency-free Engram adapter against a real server."""

from __future__ import annotations

import importlib.util
import json
import pathlib
import sys


path = pathlib.Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("engram_example", path)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

if hasattr(module, "remember_decision"):
    created_raw = module.remember_decision(f"smoke decision from {path.parent.name}")
    found_raw = module.search_memory("smoke decision")
else:
    state = {
        "query": "smoke decision",
        "decision": f"smoke decision from {path.parent.name}",
        "workspace": module.DEFAULT_WORKSPACE,
    }
    created_raw = module.remember_decision_node(state)["create_result"]
    found_raw = module.search_memory_node(state)["search_result"]

created = json.loads(created_raw)
found = json.loads(found_raw)
assert "error" not in created, created
assert "error" not in found, found
assert "result" in created and "result" in found
