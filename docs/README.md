# Engram Docs

Engram is a memory layer for AI agents and teams that need to organize proprietary context, reduce information asymmetry, and expose the same source of truth through MCP, HTTP, and local tooling.

## What Lives Where

- `ARCHITECTURE.md`: Hosted/cloud architecture reference, request flow, and tenant isolation model.
- `REFERENCE.md`: Hosted HTTP/MCP API reference, auth, tenant resolution, and error codes.
- `OPERATIONS.md`: Hosted deployment targets, alerts, backup/restore, and incident playbooks.
- `USER_GUIDE.md`: User-facing instructions for installing, connecting, and using Engram.
- `CONTROL_PLANE_SCHEMA.sql`: Postgres schema for the cloud control plane.
- `QUICKSTART.md`: Command-first local quick reference.
- `SCHEMA.md`: Local Engram SQLite schema and migrations.
- `integrations/`: MCP and ecosystem integration guides for Claude Code, Cursor, OpenAI agent workflows, and generic MCP clients.
- `awesome-list-submissions.md`: External awesome-list targets, opened PRs, and blocked/manual submissions.
- `../examples/`: Runnable MCP and ecosystem memory examples for Claude, Cursor, OpenAI Agents SDK, FastMCP, LangGraph, and CrewAI.
- `releases/`: Release-note source, next-version policy, and public-claims gate.
- `rfcs/`: Proposed design records and product-boundary RFCs.
- `strategy/`: Product and competitive strategy notes extracted from prior research.
- `USING_ENGRAM_IN_A_REPO.md`: How another repository connects to Engram through MCP, CLI, or HTTP.

## Notes

- Cloud documents describe hosted deployment patterns and planned managed-service surfaces. They do not change core Engram behavior, and public SaaS availability or SLO claims need deployment-time verification.
- Root stays reserved for repository identity, build/config entrypoints, GitHub-visible policy files, and agent context files consumed by tooling.
- Core/OSS documentation lives in the repo root when tools expect it there, or under `docs/` when it is reference material.
- The README explains the product thesis and primary workflows; this directory explains how the cloud, local schema, and repo integration pieces fit together.
