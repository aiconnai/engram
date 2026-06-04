---
name: engram-council
description: Run structured consensus with Engram's `memory_council` tool and `CouncilSkill` SDK wrappers for architecture, design, policy, security, reliability, performance, ADR, or tradeoff decisions. Use when the user asks for consensus, council review, multi-agent-style review, decision rationale, option comparison, or mentions `llm-council`, `memory_council`, or `CouncilSkill`. Do not use for simple lookup, single-answer retrieval, or execution-only coding tasks after a decision is already made.
---

# Engram Council

Use this skill to turn an ambiguous decision into a bounded council run and a concise recommendation. The installable skill name is `engram-council`; `llm-council` is the upstream council backend used by Engram's MCP tool.

## Operating Rules

- Do not fabricate a council result. If no MCP or SDK execution path is available, return the prepared council prompt and state that it was not executed.
- Ask at most one clarifying question when missing options, criteria, or persistence would make the run low quality. Otherwise make conservative assumptions and state them.
- Keep one decision per council run. Split unrelated decisions into separate runs.
- Default to `persist=false` unless the decision should survive across sessions, become an ADR/checkpoint, or guide future agents.
- Never send secrets, credentials, private keys, raw customer data, or proprietary content that the user has not approved for this council backend.
- Treat `memory_council` as potentially mutating when `persist=true`; report the target workspace and tags.

## Use / Skip

Use this skill for:

- architecture and design tradeoffs
- project policy or standards decisions
- security, reliability, performance, and operations choices
- ADR preparation or decision checkpoints
- multi-model or multi-agent review prompts where agreement and dissent matter

Skip this skill for:

- factual lookup or memory search
- implementation tasks where the decision is already settled
- broad brainstorming without a concrete choice to make
- requests that need deterministic tests or code review more than opinion synthesis

## Input Checklist

Before executing, identify:

- `decision_question`: the exact decision to make
- `options`: named choices, including "defer" if valid
- `criteria`: success criteria and ranking priorities
- `context`: relevant facts, constraints, evidence, and non-goals
- `workspace`: where a persisted result should live
- `persistence`: `persist=true` or `persist=false`
- `raw_stages`: whether stage outputs are needed for audit/debugging

If options are not explicit, infer the smallest useful set and include them in the prompt. If the request is too vague for that, ask one clarifying question.

## Council Prompt

Use a prompt shaped like this:

```text
Decision question:
<one precise question>

Options:
- A: ...
- B: ...
- Defer/no-change: ... (if valid)

Criteria, in priority order:
1. ...
2. ...
3. ...

Context and evidence:
- ...

Constraints and non-goals:
- ...

Required output:
- recommended option
- confidence: high, medium, or low
- strongest reasons
- strongest dissent or caveat
- risks that would change the decision
- next steps
```

## Execution Path

Choose the first available path:

1. If you are executing as an agent with Engram MCP access, call `memory_council` directly.
2. If you are writing integration code, use the Python or TypeScript `CouncilSkill` wrapper.
3. If neither path is available, produce the council prompt and say the council was not executed.

### MCP Arguments

Use these defaults unless the user or repo context says otherwise:

- `prompt`: the council prompt above
- `timeout_seconds`: `120` for normal decisions, up to `300` for complex reviews
- `include_raw_stages`: `false` for normal answers, `true` when auditing model disagreement
- `persist`: `false` by default
- `workspace`: repo, project, or domain workspace when `persist=true`
- `memory_tags`: include `llm-council`, `consensus`, and one domain tag such as `architecture`, `security`, or `ops`
- `council_url`: omit unless the default backend URL is wrong

Direct MCP shape:

```json
{
  "name": "memory_council",
  "arguments": {
    "prompt": "<council prompt>",
    "timeout_seconds": 120,
    "include_raw_stages": false,
    "persist": true,
    "workspace": "architecture",
    "memory_tags": ["llm-council", "consensus", "architecture"]
  }
}
```

## SDK Recipes

Use these only when adding app/repo integration code.

Python:

```python
from engram_client.integrations import CouncilSkill

council = CouncilSkill(
    client,
    default_workspace="architecture",
    default_timeout_seconds=120,
    default_include_raw_stages=False,
)

result = await council.ask_with_persistence(
    "Decision question: Should we standardize on PostgreSQL for new services?"
)
```

TypeScript:

```typescript
import { CouncilSkill } from "engram-client";

const council = new CouncilSkill(client, {
  defaultWorkspace: "architecture",
  defaultTimeoutSeconds: 120,
  defaultIncludeRawStages: false,
});

const result = await council.askWithPersistence(
  "Decision question: Should we standardize on PostgreSQL for new services?"
);
```

## Interpret Results

Expected successful fields include:

- `final_answer`: primary answer to summarize
- `final_model`: final model or arbiter, when available
- `stage1`, `stage2`, `stage3` or stage counts, depending on `include_raw_stages`
- `memory_id`, when persistence succeeded
- `warning`, when the council completed but persistence failed
- `error`, when the council did not complete

If `error` exists, do not present a recommendation as council-backed. Report the failure and provide the prepared prompt or a narrowed retry.

If `warning` exists, treat the council result as valid but clearly state that persistence failed.

## Response Format

Return this structure:

```markdown
**Council Result**
- **Decision:** <recommended option>
- **Confidence:** high | medium | low
- **Rationale:** <2-4 concise reasons>
- **Dissent/Caveat:** <strongest objection or uncertainty>
- **Risks:** <what could change the answer>
- **Persistence:** persisted to `<workspace>` as memory `<id>` | not persisted | persistence failed: `<warning>`
- **Next steps:** <1-3 concrete actions>
```

Add a short `Assumptions:` line before the result if you inferred options, criteria, workspace, or persistence.

## Failure Handling

- Empty or vague prompt: ask for the decision question and options.
- Backend unreachable: check MCP transport, `LLM_COUNCIL_URL`, `council_url`, auth, and tenant/workspace configuration.
- Timeout: reduce context to one decision and the top criteria, or retry with a higher `timeout_seconds`.
- Low-quality result: rerun with explicit options, criteria priority, and non-goals.
- Sensitive input: stop and ask for a sanitized version or approval before sending it to the council backend.
