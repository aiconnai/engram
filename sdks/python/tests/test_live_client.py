"""Live package-contract tests for the installed Python SDK wheel."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import cast

import pytest

import engram_client
from engram_client import EngramClient, EngramError


def live_config() -> tuple[str, str, str, str]:
    """Read the disposable server contract injected by the live test driver."""
    names = (
        "ENGRAM_LIVE_BASE_URL",
        "ENGRAM_LIVE_API_KEY",
        "ENGRAM_LIVE_TENANT",
        "ENGRAM_LIVE_VENV",
    )
    values = tuple(os.environ.get(name) for name in names)
    if any(value is None for value in values):
        pytest.skip(
            "live SDK server is not configured; use scripts/test-python-sdk-live.sh"
        )
    return cast(tuple[str, str, str, str], values)


def assert_installed_wheel(venv: str) -> None:
    """Reject accidental imports from the SDK source tree."""
    package_path = Path(engram_client.__file__).resolve()
    assert package_path.is_relative_to(Path(venv).resolve()), package_path
    assert "sdks/python/engram_client" not in package_path.as_posix()


@pytest.mark.asyncio
async def test_live_public_memory_contract() -> None:
    base_url, api_key, tenant, venv = live_config()
    if os.environ.get("ENGRAM_LIVE_SCENARIO", "happy") != "happy":
        pytest.skip("happy-path contract is not selected")
    assert_installed_wheel(venv)

    content = "Python SDK live wheel remembers the indigo launch contract"
    updated_content = f"{content} updated"
    client = EngramClient(base_url, api_key, tenant, timeout=5.0)
    try:
        created = await client.create(
            content,
            workspace=tenant,
            tags=["python-sdk-live"],
            metadata={"contract": "installed-wheel"},
        )
        memory_id = created["id"]
        assert created["content"] == content

        fetched = await client.get(memory_id)
        assert fetched["id"] == memory_id
        assert fetched["content"] == content

        listed = await client.list(workspace=tenant)
        assert any(memory["id"] == memory_id for memory in listed)

        searched = await client.search("indigo launch contract", workspace=tenant)
        assert str(memory_id) in json.dumps(searched)
        assert content in json.dumps(searched)

        updated = await client.update(memory_id, content=updated_content)
        assert updated["content"] == updated_content

        deleted = await client.delete(memory_id)
        assert deleted["deleted"] == memory_id
        with pytest.raises(EngramError):
            await client.get(memory_id)
    finally:
        await client.close()

    with pytest.raises(EngramError, match="closed"):
        await client.list(workspace=tenant)

    async with EngramClient(base_url, api_key, tenant, timeout=5.0) as managed:
        context_memory = await managed.create(
            "Python SDK async context manager contract",
            workspace=tenant,
        )
        context_id = context_memory["id"]
        assert (await managed.get(context_id))["id"] == context_id
        await managed.delete(context_id)
    with pytest.raises(EngramError, match="closed"):
        await managed.get(context_id)


@pytest.mark.asyncio
async def test_live_wrong_bearer_is_typed() -> None:
    base_url, api_key, tenant, venv = live_config()
    if os.environ.get("ENGRAM_LIVE_SCENARIO") != "wrong_bearer":
        pytest.skip("wrong-bearer contract is not selected")
    assert_installed_wheel(venv)

    async with EngramClient(base_url, api_key, tenant, timeout=5.0) as client:
        with pytest.raises(EngramError, match="HTTP 401"):
            await client.list(workspace=tenant)


@pytest.mark.asyncio
async def test_live_killed_server_is_typed() -> None:
    base_url, api_key, tenant, venv = live_config()
    if os.environ.get("ENGRAM_LIVE_SCENARIO") != "killed_server":
        pytest.skip("killed-server contract is not selected")
    assert_installed_wheel(venv)

    async with EngramClient(base_url, api_key, tenant, timeout=1.0) as client:
        with pytest.raises(EngramError, match="Engram request failed"):
            await client.list(workspace=tenant)
