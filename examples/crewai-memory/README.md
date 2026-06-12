# CrewAI Memory Example

The Python SDK includes CrewAI-style memory adapters:

- `EngramShortTermMemory`
- `EngramLongTermMemory`
- `EngramEntityMemory`

## Sketch

```python
from engram_client import EngramClient
from engram_client.integrations.crewai import (
    EngramEntityMemory,
    EngramLongTermMemory,
    EngramShortTermMemory,
)

client = EngramClient(base_url="https://your-engram-api", api_key="ek_...", tenant="team")

short_term = EngramShortTermMemory(client, workspace="crewai-stm")
long_term = EngramLongTermMemory(client, crew_name="research")
entities = EngramEntityMemory(client)

short_term.save("last-task", "Summarized customer interviews")
long_term.save("pricing-finding", "Enterprise buyers asked for audit logs")
entities.save_entity("Acme Corp", "company", "Enterprise prospect")
```

## Limitation

This adapter lives in the Python SDK. It assumes your CrewAI workflow owns when
to save, search, and clear memory.
