from __future__ import annotations

import re
from pathlib import Path
from typing import Final, Union

JsonScalar = Union[str, int, float, bool, None]
JsonValue = Union[JsonScalar, list["JsonValue"], dict[str, "JsonValue"]]
JsonObject = dict[str, JsonValue]

SCHEMA_PATH: Final = Path("docs/quality/baseline.schema.json")
SCHEMA_VERSION: Final = "engram.quality-baseline.v1"
REQUIRED_METRICS: Final = ("recall@10", "mrr", "ndcg@10")
REQUIRED_TOP_LEVEL: Final = (
    "schema_version",
    "source_revision",
    "generated_at",
    "deterministic_seed",
    "corpus",
    "metrics",
    "benchmark_evidence",
)
BENCHMARK_LINE: Final = re.compile(
    r"^(?P<name>[A-Za-z0-9_./ -]+):\s+(?P<value>[0-9]+(?:\.[0-9]+)?)\s+(?P<unit>ns|us|µs|ms|s)$"
)
SHA_RE: Final = re.compile(r"^[0-9a-f]{40}$")
UTC_TIMESTAMP_RE: Final = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
CORPUS_FIELDS: Final = ("name", "version", "fixture_path", "schema", "memory_count", "query_count")
CORPUS_SCHEMA_FIELDS: Final = ("memory_fields", "query_fields", "relevance_fields")
EVIDENCE_FIELDS: Final = ("criterion_baseline", "dream_eval_runbook")


class QualityBaselineError(Exception):
    __slots__ = ("reason",)

    def __init__(self, reason: str) -> None:
        super().__init__(reason)
        self.reason = reason

    def __str__(self) -> str:
        return self.reason


class CriterionMetric(tuple):
    __slots__ = ()

    def __new__(cls, name: str, value: float, unit: str) -> "CriterionMetric":
        return tuple.__new__(cls, (name, value, unit))

    @property
    def name(self) -> str:
        return self[0]

    @property
    def value(self) -> float:
        return self[1]

    @property
    def unit(self) -> str:
        return self[2]


class ValidationResult(tuple):
    __slots__ = ()

    def __new__(cls, artifact_kind: str, metric_names: tuple[str, ...]) -> "ValidationResult":
        return tuple.__new__(cls, (artifact_kind, metric_names))

    @property
    def artifact_kind(self) -> str:
        return self[0]

    @property
    def metric_names(self) -> tuple[str, ...]:
        return self[1]
