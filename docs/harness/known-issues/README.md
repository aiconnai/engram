# Known Issues for Documented Sensor Exclusions

This directory holds short, dated documents that justify temporary exclusion of a specific sensor in `sensors.sh`.

**Only** for external dependency outages (embedding providers, third-party APIs, GUI-dependent watcher tests, etc.).

Each file must follow the naming `YYYY-MM-DD-<slug>.md` and be referenced explicitly with `--known-issue` + `--reason` + prior registration in the active progress log.

Example structure:

```markdown
# 2026-05-30-cohere-outage

**Sensor**: embedding-api-smoke
**Date**: 2026-05-30
**Reason**: Cohere API returning 5xx for all requests in the last 4 hours. Confirmed via status page.

**Impact**: Only affects optional smoke test that calls the live embedding provider. Core embedding code path (with local-embeddings feature) is unaffected.

**Mitigation**: Run with `--exclude-sensor embedding-api-smoke ...` only for this task. Production closure requires a clean run once the outage is resolved.

**Recorded in**: docs/harness/progress/2026-05-30-xxx.md and docs/harness/progress.md
```

Never use this mechanism to hide real problems in the codebase.

## Outra categoria: achados de review adiados

Este diretório também pode guardar a **triagem de achados de review adiados** (não são exclusões de
sensor e nunca servem como `--known-issue`). Eles ficam em arquivos com o mesmo nome
`YYYY-MM-DD-<slug>.md`, declaram "Tipo: achados de review adiados" no topo e carregam status
explícito (aberto, em andamento, parcial, corrigido com SHA). Primeiro caso:
[`2026-10-26-deferred-review-minors.md`](./2026-10-26-deferred-review-minors.md).
