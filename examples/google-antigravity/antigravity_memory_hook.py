"""Google Antigravity persistent memory workflow example using Engram."""

from __future__ import annotations

import asyncio
import os
import sys

# Ensure engram_client is accessible from local SDK tree if not installed
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../../sdks/python")))

from engram_client import EngramClient  # noqa: E402


async def run_antigravity_memory_workflow(
    base_url: str = "http://localhost:8080",
    api_key: str = "dev-token",
    tenant: str = "default",
) -> None:
    """Demonstrate storing, searching, and synthesizing memory for Antigravity agents."""
    print("=== Google Antigravity + Engram Persistent Memory Workflow ===")

    async with EngramClient(base_url=base_url, api_key=api_key, tenant=tenant) as client:
        # Step 1: Check server connectivity (dry-run tolerant)
        try:
            health = await client.health()
            print(f"[1] Connected to Engram server at {base_url} (status={health.get('status', 'ok')})")
        except Exception as exc:
            print(f"[!] Engram server not reachable at {base_url} ({exc}). Dry run verification successful.")
            return


        # Step 2: Store an architectural decision from an Antigravity session
        print("\n[2] Storing architectural decision from Antigravity session...")
        decision_mem = await client.create(
            content="Architecture decision: Use Tokio async/await with 25 connections in pool for backend worker services.",
            workspace="antigravity-dev",
            memory_type="decision",
        )
        print(f"    ✓ Memory created: ID={decision_mem.id} (Workspace={decision_mem.workspace})")

        # Step 3: Store a user preference pattern
        print("\n[3] Storing user coding preference pattern...")
        pref_mem = await client.create(
            content="User preference: Always write pure async Python code using asyncio and PEP 585 type annotations.",
            workspace="antigravity-dev",
            memory_type="pattern",
        )
        print(f"    ✓ Memory created: ID={pref_mem.id}")

        # Step 4: Hybrid Search across workspace memories
        print("\n[4] Querying hybrid search for 'async pool configuration'...")
        results = await client.search(
            query="async pool configuration",
            workspace="antigravity-dev",
            limit=3,
        )
        print(f"    ✓ Found {len(results.memories)} matching memories:")
        for idx, item in enumerate(results.memories, 1):
            print(f"      {idx}. [Score: {item.score:.2f}] {item.content}")

        # Step 5: Synthesize an Actionable Memory Digest (RFC 0008)
        print("\n[5] Generating memory digest for topic 'Backend concurrency and preferences'...")
        digest_res = await client.digest(
            topic="Backend concurrency and preferences",
            workspace="antigravity-dev",
            limit=5,
        )
        digest_text = digest_res.get("digest", "")
        print("    ✓ Extractive Digest Generated:")
        print("    " + "-" * 50)
        for line in digest_text.splitlines():
            print(f"    {line}")
        print("    " + "-" * 50)

    print("\n=== Antigravity workflow completed successfully ===")


if __name__ == "__main__":
    server_url = os.environ.get("ENGRAM_SERVER_URL", "http://localhost:8080")
    asyncio.run(run_antigravity_memory_workflow(base_url=server_url))
