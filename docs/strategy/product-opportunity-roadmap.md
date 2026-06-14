# Product Opportunity Roadmap

This is a strategy input, not the canonical delivery roadmap. The shipped-phase
record remains in [../ROADMAP.md](../ROADMAP.md), and concrete implementation
work should move through RFCs, issues, and harness plans.

## Near-Term Themes

### Developer Experience and Adoption

Goal: reduce time from installation to first useful memory.

Possible slices:

- One-command or low-friction launch paths for local MCP use.
- Copy-pasteable setup guides for common MCP hosts and agent runtimes.
- Framework adapters where they remove real adoption friction.
- Reproducible examples that exercise storage, search, and memory retrieval
  end to end.
- Published benchmark commands and raw data, once verified.

Risks:

- Shipping wrappers before core workflows are stable creates support load.
- Hardcoding tool counts or current platform claims makes public docs stale.

### Human Inspection and Capture

Goal: make memory inspectable, searchable, and correctable by humans.

Possible slices:

- Lightweight dashboard for search, list, inspect, and edit workflows.
- Knowledge graph view for memory relationships and provenance.
- Browser or URL capture workflow for research notes and documentation.
- Markdown and knowledge-base import/export flows with dry-run previews.

Risks:

- A large UI can distract from Engram's core memory substrate.
- Capture workflows can ingest low-quality content unless deduplication,
  metadata, and provenance are strict.

### Proactive Memory Quality

Goal: improve what gets stored before retrieval tries to use it.

Possible slices:

- Proactive extraction from sessions or documents with source links.
- Self-questioning or evidence-check steps before accepting derived facts.
- Contradiction and freshness review workflows with explicit apply boundaries.
- Daily or topic digest surfaces that summarize without mutating canonical
  memory.

Risks:

- LLM-assisted extraction can hallucinate unless every claim is grounded in
  source memory or source artifacts.
- Automatic consolidation must preserve provenance and avoid silent canonical
  rewrites.

### Team and Enterprise Readiness

Goal: make shared memory operationally credible for teams.

Possible slices:

- Clear deployment guides for local, private server, and managed modes.
- Access-control and sharing examples tied to real team workflows.
- Audit, retention, and attestation documentation for regulated users.
- Connector framework design for Slack, GitHub, Notion, Google Drive, or
  Confluence style sources.

Risks:

- Compliance and enterprise claims require evidence, not aspiration.
- Connector work can introduce credential and rate-limit complexity.

## Candidate Backlog

| Candidate | Source inspiration | Extraction status |
|---|---|---|
| Query rewriting and expansion | Developer memory APIs and visual search systems | Keep as issue candidate. |
| User or entity profile summaries | Personalization-focused memory products | Keep as RFC candidate because schema and privacy matter. |
| Dashboard and graph inspection | Consumer and graph-first memory tools | Keep as product slice, scoped to inspection first. |
| Framework adapters | Developer-first memory APIs | Extract only when a target adapter has a maintainer and tests. |
| Proactive extraction with verification | Research memory systems | Extract as a research-backed RFC before implementation. |
| URL and knowledge-base ingestion | Consumer capture products | Extract in small dry-run-first increments. |
| Tool versioning and deprecation | Mature API platforms | Extract when MCP surface changes require migration support. |
| Multi-agent handoff protocol | Federation and orchestration systems | Keep as reference until current sharing primitives need expansion. |

## Roadmap Hygiene

- Do not publish this file as a committed release timeline.
- Do not include exact competitor popularity metrics without a dated source.
- Do not claim benchmark superiority until the benchmark harness and raw data
  are committed and reproducible.
- Keep current product state linked to generated references and source docs.
