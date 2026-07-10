from __future__ import annotations

import json
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Sequence

from quality_baseline.contract import (
    BENCHMARK_LINE,
    CORPUS_FIELDS,
    CORPUS_SCHEMA_FIELDS,
    EVIDENCE_FIELDS,
    REQUIRED_METRICS,
    REQUIRED_TOP_LEVEL,
    SCHEMA_VERSION,
    SHA_RE,
    UTC_TIMESTAMP_RE,
    CriterionMetric,
    JsonObject,
    JsonValue,
    QualityBaselineError,
    ValidationResult,
)


def read_schema(path: Path) -> JsonObject:
    if not path.exists():
        raise QualityBaselineError(f"missing_schema:{path}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise QualityBaselineError(f"malformed_schema_json:{path}:{error.msg}") from error
    if not isinstance(data, dict):
        raise QualityBaselineError(f"malformed_schema_json:{path}:top_level_not_object")
    return data


def ensure_schema_contract(schema: JsonObject) -> None:
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        raise QualityBaselineError("schema_missing:properties")
    metrics = properties.get("metrics")
    if not isinstance(metrics, dict):
        raise QualityBaselineError("schema_missing:metrics")
    required = metrics.get("required")
    if not isinstance(required, list):
        raise QualityBaselineError("schema_missing:metrics.required")
    missing = tuple(metric for metric in REQUIRED_METRICS if metric not in required)
    if missing:
        raise QualityBaselineError(f"schema_missing_metrics:{','.join(missing)}")


def validate_criterion_text(path: Path) -> ValidationResult:
    if not path.exists():
        raise QualityBaselineError(f"missing_artifact:{path}")
    lines = tuple(line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip())
    if not lines:
        raise QualityBaselineError("criterion_text_missing:Baseline header")
    if lines[0] != "Baseline:":
        raise QualityBaselineError("criterion_text_missing:Baseline header")

    metrics = tuple(parse_criterion_metric(line) for line in lines[1:])
    if not metrics:
        raise QualityBaselineError("criterion_text_missing:benchmark metric line")
    metric_names = tuple(metric.name for metric in metrics)
    return ValidationResult(artifact_kind="criterion_text", metric_names=metric_names)


def parse_criterion_metric(line: str) -> CriterionMetric:
    match = BENCHMARK_LINE.fullmatch(line)
    if match is None:
        raise QualityBaselineError(f"criterion_text_malformed_metric:{line}")
    value = float(match.group("value"))
    if value <= 0.0:
        raise QualityBaselineError(f"criterion_text_non_positive_metric:{match.group('name')}")
    return CriterionMetric(name=match.group("name"), value=value, unit=match.group("unit"))


def validate_retrieval_json(path: Path, enforce_head: bool) -> ValidationResult:
    data = read_artifact_json(path)
    missing = missing_retrieval_fields(data)
    if missing:
        raise QualityBaselineError(f"retrieval_baseline_missing:{','.join(missing)}")
    reject_extra_fields(data, REQUIRED_TOP_LEVEL, "retrieval_baseline")
    validate_source_revision(data["source_revision"], enforce_head)
    validate_timestamp(data["generated_at"])
    validate_nonnegative_int(data["deterministic_seed"], "deterministic_seed")
    validate_corpus(data["corpus"])
    validate_evidence(data["benchmark_evidence"])
    validate_metrics(data["metrics"])

    if data["schema_version"] != SCHEMA_VERSION:
        raise QualityBaselineError("retrieval_baseline_invalid:schema_version")
    return ValidationResult(artifact_kind="retrieval_json", metric_names=tuple(REQUIRED_METRICS))


def read_artifact_json(path: Path) -> JsonObject:
    if not path.exists():
        raise QualityBaselineError(f"missing_artifact:{path}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise QualityBaselineError(f"malformed_input_json:{path}:{error.msg}") from error
    if not isinstance(data, dict):
        raise QualityBaselineError("retrieval_baseline_missing:top_level_object")
    return data


def validate_source_revision(value: JsonValue, enforce_head: bool) -> None:
    if not isinstance(value, str) or SHA_RE.fullmatch(value) is None:
        raise QualityBaselineError("retrieval_baseline_invalid:source_revision")
    if enforce_head and value != git_head():
        raise QualityBaselineError("retrieval_baseline_stale:source_revision")


def validate_timestamp(value: JsonValue) -> None:
    if not isinstance(value, str) or UTC_TIMESTAMP_RE.fullmatch(value) is None:
        raise QualityBaselineError("retrieval_baseline_invalid:generated_at")
    try:
        datetime.fromisoformat(f"{value[:-1]}+00:00")
    except ValueError as error:
        raise QualityBaselineError("retrieval_baseline_invalid:generated_at") from error


def validate_metrics(value: JsonValue) -> None:
    metrics = require_object(value, "metrics")
    reject_extra_fields(metrics, REQUIRED_METRICS, "metrics")
    for metric_name in REQUIRED_METRICS:
        value = metrics[metric_name]
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise QualityBaselineError(f"retrieval_baseline_invalid:metrics.{metric_name}")
        if not 0.0 <= float(value) <= 1.0:
            raise QualityBaselineError(f"retrieval_baseline_invalid:metrics.{metric_name}")


def validate_nonnegative_int(value: JsonValue, field: str) -> None:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise QualityBaselineError(f"retrieval_baseline_invalid:{field}")


def validate_positive_int(value: JsonValue, field: str) -> None:
    if isinstance(value, bool) or not isinstance(value, int) or value < 1:
        raise QualityBaselineError(f"retrieval_baseline_invalid:{field}")


def validate_corpus(value: JsonValue) -> None:
    corpus = require_object(value, "corpus")
    reject_missing_fields(corpus, CORPUS_FIELDS, "corpus")
    reject_extra_fields(corpus, CORPUS_FIELDS, "corpus")
    for field in ("name", "version", "fixture_path"):
        require_nonempty_string(corpus[field], f"corpus.{field}")
    validate_positive_int(corpus["memory_count"], "corpus.memory_count")
    validate_positive_int(corpus["query_count"], "corpus.query_count")
    schema = require_object(corpus["schema"], "corpus.schema")
    reject_missing_fields(schema, CORPUS_SCHEMA_FIELDS, "corpus.schema")
    reject_extra_fields(schema, CORPUS_SCHEMA_FIELDS, "corpus.schema")
    for field in CORPUS_SCHEMA_FIELDS:
        require_string_list(schema[field], f"corpus.schema.{field}")


def validate_evidence(value: JsonValue) -> None:
    evidence = require_object(value, "benchmark_evidence")
    reject_missing_fields(evidence, EVIDENCE_FIELDS, "benchmark_evidence")
    reject_extra_fields(evidence, EVIDENCE_FIELDS, "benchmark_evidence")
    for field in EVIDENCE_FIELDS:
        require_nonempty_string(evidence[field], f"benchmark_evidence.{field}")


def require_object(value: JsonValue, field: str) -> JsonObject:
    if not isinstance(value, dict):
        raise QualityBaselineError(f"retrieval_baseline_invalid:{field}")
    return value


def require_nonempty_string(value: JsonValue, field: str) -> None:
    if not isinstance(value, str) or not value:
        raise QualityBaselineError(f"retrieval_baseline_invalid:{field}")


def require_string_list(value: JsonValue, field: str) -> None:
    if not isinstance(value, list) or not value:
        raise QualityBaselineError(f"retrieval_baseline_invalid:{field}")
    for item in value:
        if not isinstance(item, str) or not item:
            raise QualityBaselineError(f"retrieval_baseline_invalid:{field}")
    if len(set(value)) != len(value):
        raise QualityBaselineError(f"retrieval_baseline_invalid:{field}")


def reject_extra_fields(value: JsonObject, allowed: Sequence[str], field: str) -> None:
    extra = sorted(set(value) - set(allowed))
    if extra:
        raise QualityBaselineError(f"retrieval_baseline_unknown:{field}.{','.join(extra)}")


def reject_missing_fields(value: JsonObject, required: Sequence[str], field: str) -> None:
    missing = tuple(f"{field}.{name}" for name in required if name not in value)
    if missing:
        raise QualityBaselineError(f"retrieval_baseline_missing:{','.join(missing)}")


def missing_retrieval_fields(data: JsonObject) -> tuple[str, ...]:
    missing: list[str] = []
    for field in REQUIRED_TOP_LEVEL:
        if field not in data:
            missing.append(field)
    metrics = data.get("metrics")
    if isinstance(metrics, dict):
        for metric_name in REQUIRED_METRICS:
            if metric_name not in metrics:
                missing.append(f"metrics.{metric_name}")
    elif "metrics" not in missing:
        missing.extend(f"metrics.{metric_name}" for metric_name in REQUIRED_METRICS)
    return tuple(sorted(missing))


def git_head() -> str:
    try:
        completed = subprocess.run(["git", "rev-parse", "HEAD"], check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as error:
        raise QualityBaselineError("git_head_unavailable") from error
    return completed.stdout.strip()
