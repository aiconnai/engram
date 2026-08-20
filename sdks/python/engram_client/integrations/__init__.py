"""Engram integrations with popular AI frameworks."""

from engram_client.integrations.crewai import (
    EngramEntityMemory,
    EngramLongTermMemory,
    EngramShortTermMemory,
)
from engram_client.integrations.langchain import EngramChatMessageHistory, EngramVectorStore
from engram_client.integrations.council import CouncilSkill
from engram_client.integrations.openai_threads import EngramThreadStore

try:
    from engram_client.integrations.llamaindex import (
        EngramChatStore,
        EngramDocumentStore,
        EngramLlamaIndexVectorStore,
    )
except ImportError:
    pass

__all__ = [
    "EngramChatMessageHistory",
    "EngramVectorStore",
    "EngramShortTermMemory",
    "CouncilSkill",
    "EngramLongTermMemory",
    "EngramEntityMemory",
    "EngramThreadStore",
    "EngramDocumentStore",
    "EngramLlamaIndexVectorStore",
    "EngramChatStore",
]
