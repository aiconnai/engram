from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path
from typing import Optional, Sequence

from quality_baseline.contract import (
    JsonObject,
    JsonValue,
    SCHEMA_PATH,
    SCHEMA_VERSION,
    QualityBaselineError,
    ValidationResult,
)
from quality_baseline.validation import (
    ensure_schema_contract,
    read_schema,
    validate_criterion_text,
    validate_retrieval_json,
)


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = tuple(sys.argv[1:] if argv is None else argv)
    try:
        result = run(args)
    except QualityBaselineError as error:
        print(f"QUALITY_BASELINE_INVALID: {error}", file=sys.stderr)
        return 1

    print(
        "QUALITY_BASELINE_OK: "
        f"artifact_kind={result.artifact_kind} metrics={','.join(result.metric_names)}"
    )
    return 0


def run(args: Sequence[str]) -> ValidationResult:
    if args == ("--self-test-invalid",):
        return run_invalid_self_test()
    if not args:
        raise QualityBaselineError(
            "usage: check-quality-baseline.py <artifact> | --self-test-invalid"
        )
    if len(args) > 2:
        raise QualityBaselineError("usage: check-quality-baseline.py [--enforce-head] <artifact>")

    enforce_head = False
    paths = tuple(args)
    if paths[0] == "--enforce-head":
        enforce_head = True
        paths = paths[1:]
    if len(paths) != 1:
        raise QualityBaselineError("usage: check-quality-baseline.py [--enforce-head] <artifact>")

    schema = read_schema(SCHEMA_PATH)
    ensure_schema_contract(schema)
    artifact_path = Path(paths[0])
    if artifact_path.suffix == ".json":
        return validate_retrieval_json(artifact_path, enforce_head)
    return validate_criterion_text(artifact_path)


def run_invalid_self_test() -> ValidationResult:
    invalid = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": "2026-07-10T00:00:00Z",
        "corpus": self_test_corpus(),
        "metrics": {},
        "benchmark_evidence": self_test_evidence(),
    }
    nested_invalid = {
        "schema_version": SCHEMA_VERSION,
        "source_revision": "0" * 40,
        "generated_at": "not-a-date",
        "deterministic_seed": -1,
        "corpus": "not-an-object",
        "metrics": {"recall@10": 1.0, "mrr": 1.0, "ndcg@10": 1.0},
        "benchmark_evidence": {},
    }
    with tempfile.TemporaryDirectory() as tmpdir:
        invalid_path = Path(tmpdir).joinpath("invalid.json")
        invalid_path.write_text(json.dumps(invalid), encoding="utf-8")
        nested_path = Path(tmpdir).joinpath("nested-invalid.json")
        nested_path.write_text(json.dumps(nested_invalid), encoding="utf-8")
        offset_path = Path(tmpdir).joinpath("offset-generated-at.json")
        offset_payload = self_test_valid_retrieval_baseline()
        offset_payload["generated_at"] = "2026-07-10T02:00:00+02:00"
        offset_path.write_text(json.dumps(offset_payload), encoding="utf-8")
        space_path = Path(tmpdir).joinpath("space-generated-at.json")
        space_payload = self_test_valid_retrieval_baseline()
        space_payload["generated_at"] = "2026-07-10 00:00:00Z"
        space_path.write_text(json.dumps(space_payload), encoding="utf-8")
        zero_offset_path = Path(tmpdir).joinpath("zero-offset-generated-at.json")
        zero_offset_payload = self_test_valid_retrieval_baseline()
        zero_offset_payload["generated_at"] = "2026-07-10T00:00:00+00:00"
        zero_offset_path.write_text(json.dumps(zero_offset_payload), encoding="utf-8")
        canonical_path = Path(tmpdir).joinpath("canonical-generated-at.json")
        canonical_path.write_text(json.dumps(self_test_valid_retrieval_baseline()), encoding="utf-8")
        missing = expect_invalid(invalid_path, "retrieval_baseline_missing:")
        expect_invalid(nested_path, "retrieval_baseline_invalid:generated_at")
        expect_invalid(offset_path, "retrieval_baseline_invalid:generated_at")
        expect_invalid(space_path, "retrieval_baseline_invalid:generated_at")
        expect_invalid(zero_offset_path, "retrieval_baseline_invalid:generated_at")
        for field in ("memory_fields", "query_fields", "relevance_fields"):
            for invalid_item in ({"not": "a-string"}, ["not-a-string"]):
                item_path = Path(tmpdir).joinpath(f"{field}-invalid-item.json")
                item_payload = self_test_field_list_item_payload(field, invalid_item)
                item_path.write_text(json.dumps(item_payload), encoding="utf-8")
                expect_invalid(item_path, f"retrieval_baseline_invalid:corpus.schema.{field}")
        validate_retrieval_json(canonical_path, enforce_head=False)
    required_missing = (
        "deterministic_seed",
        "metrics.mrr",
        "metrics.ndcg@10",
        "metrics.recall@10",
        "source_revision",
    )
    absent = tuple(field for field in required_missing if field not in missing)
    if absent:
        raise QualityBaselineError(f"self_test_did_not_name_missing_fields:{','.join(absent)}")
    print(f"INVALID_SCHEMA_REJECTED: retrieval_baseline_missing:{','.join(missing)}")
    return ValidationResult(artifact_kind="self_test_invalid", metric_names=required_missing)


def self_test_valid_retrieval_baseline() -> JsonObject:
    return {
        "schema_version": SCHEMA_VERSION,
        "source_revision": "0" * 40,
        "generated_at": "2026-07-10T00:00:00Z",
        "deterministic_seed": 0,
        "corpus": self_test_corpus(),
        "metrics": {"recall@10": 1.0, "mrr": 1.0, "ndcg@10": 1.0},
        "benchmark_evidence": self_test_evidence(),
    }


def self_test_field_list_item_payload(field: str, invalid_item: JsonValue) -> JsonObject:
    payload = self_test_valid_retrieval_baseline()
    corpus = payload["corpus"]
    if not isinstance(corpus, dict):
        raise QualityBaselineError("self_test_wrong_corpus_shape")
    schema = corpus["schema"]
    if not isinstance(schema, dict):
        raise QualityBaselineError("self_test_wrong_schema_shape")
    schema[field] = [invalid_item]
    return payload


def self_test_corpus() -> JsonObject:
    return {
        "name": "self-test",
        "version": "0",
        "fixture_path": "docs/quality/fixtures/self-test.json",
        "schema": {
            "memory_fields": ["id", "content"],
            "query_fields": ["id", "query"],
            "relevance_fields": ["query_id", "memory_id", "grade"],
        },
        "memory_count": 1,
        "query_count": 1,
    }


def self_test_evidence() -> dict[str, str]:
    return {
        "criterion_baseline": "benches/results/benchmark_baseline.txt",
        "dream_eval_runbook": "docs/DREAM_SNAPSHOT_EVALS.md",
    }


def expect_invalid(path: Path, prefix: str) -> tuple[str, ...]:
    try:
        validate_retrieval_json(path, enforce_head=False)
    except QualityBaselineError as error:
        reason = str(error)
        if not reason.startswith(prefix):
            raise QualityBaselineError(f"self_test_wrong_error:{reason}") from error
        if reason.startswith("retrieval_baseline_missing:"):
            return tuple(reason.split(":", 1)[1].split(","))
        return ()
    raise QualityBaselineError("self_test_invalid_fixture_was_accepted")
