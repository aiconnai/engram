# Engram Docs

## What Lives Where

- `ARCHITECTURE.md`: Cloud/hosted architecture, request flow, and tenant isolation model.
- `REFERENCE.md`: Cloud HTTP/MCP API reference, auth, tenant resolution, and error codes.
- `OPERATIONS.md`: SLOs, alerts, backup/restore, and incident playbooks.
- `CONTROL_PLANE_SCHEMA.sql`: Postgres schema for the cloud control plane.
- `SCHEMA.md`: Local Engram SQLite schema and migrations.
- `rfcs/`: Proposed design records and product-boundary RFCs.
- `USING_ENGRAM_IN_A_REPO.md`: How another repository connects to Engram through MCP, CLI, or HTTP.

## Notes

- Cloud documents describe Engram Cloud (SaaS). They do not change core Engram behavior.
- Core/OSS documentation remains in the repo root and `docs/SCHEMA.md`.
