# Engram Coding Guidelines

## Architecture Guidelines

### Layered Architecture
Engram follows a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────┐
│         Interface Layer                  │
│  MCP • REST API • WebSocket • CLI       │
├─────────────────────────────────────────┤
│         Intelligence Layer               │
│  Salience • Quality • Entities          │
├─────────────────────────────────────────┤
│         Search Layer                     │
│  BM25 • Vectors • Fuzzy • RRF           │
├─────────────────────────────────────────┤
│         Storage Layer                    │
│  SQLite • WAL • S3/R2 Sync             │
└─────────────────────────────────────────┘
```

### Module Responsibilities

#### `storage/`
- Database operations and migrations
- Connection pooling
- Schema versioning (`SCHEMA_VERSION` in `migrations.rs`)
- **Rule**: Always update tests when changing schema version

#### `search/`
- Hybrid search implementation
- Embedding generation and caching
- Search result ranking and fusion

#### `hooks/`
- Lifecycle hooks (session end, etc.)
- Must implement `Send + Sync` for thread safety
- Use `create_handler()` pattern for hook registration

#### `mcp/`
- Model Context Protocol implementation
- Tool definitions and handlers
- JSON-RPC 2.0 compliance

## Coding Standards

### Rust
```rust
// 1. Module docs
//! Brief module description

// 2. Imports (std, external, internal)
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::Result;

// 3. Constants
const MAX_RETRIES: u32 = 3;

// 4. Type definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct Memory {
    pub id: i64,
    pub content: String,
    // ...
}

// 5. Implementations
impl Memory {
    /// Creates a new memory with validation.
    ///
    /// # Errors
    /// Returns `EngramError` if content is empty or too long.
    pub fn new(content: &str) -> Result<Self> {
        // implementation
    }
}

// 6. Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        // test code
    }
}
```

### Python (SDK)
```python
from typing import Any, Dict, Optional

class EngramClient:
    """Client for Engram memory API.

    Async context manager for proper lifecycle management.

    Example:
        >>> async with EngramClient(...) as client:
        ...     memory = await client.create("Hello")
    """

    async def __aenter__(self) -> "EngramClient":
        """Initialize HTTP client."""
        self._client = httpx.AsyncClient(...)
        return self

    async def create(
        self,
        content: str,
        filter_: Optional[Dict] = None,  # Note: filter_ to avoid shadowing built-in
    ) -> Dict[str, Any]:
        """Create a new memory.

        Args:
            content: Memory content text.
            filter_: Optional metadata filters (mapped to "filter" in API).

        Returns:
            Created memory data.
        """
        # implementation
```

### TypeScript (SDK)
```typescript
/** Configuration for EngramClient */
interface EngramConfig {
  baseUrl: string;
  apiKey: string;
  timeout?: number;
}

/** Client for Engram memory API */
export class EngramClient {
  private config: EngramConfig;

  /** Creates a new memory */
  async create(content: string, options?: CreateOptions): Promise<MemoryResult> {
    // implementation with proper TSDoc
  }
}
```

## Performance Guidelines

### Database
- Use connection pooling (`r2d2` or similar)
- Prepared statements for repeated queries
- WAL mode for better concurrent reads

### Search
- Cache embeddings when possible
- Use RRF (Reciprocal Rank Fusion) for hybrid results
- Limit result sets appropriately

### Memory
- Avoid cloning large strings unnecessarily
- Use `&str` instead of `String` when possible
- Consider zero-copy deserialization

## Security Guidelines

### Input Validation
- Validate all user input (length, content type)
- Sanitize strings before database storage
- Check permissions for scoped operations

### API Security
- Use Bearer token authentication
- Validate tenant slug in multi-tenant mode
- HTTPS only for cloud operations

### Secrets
- Never commit API keys or tokens
- Use environment variables for configuration
- Encrypt sensitive data at rest (AES-256-GCM for cloud sync)

## Testing Strategy

### Unit Tests
- Test each function/method in isolation
- Mock external dependencies
- Cover success and error cases

### Integration Tests
- Located in `tests/` directory
- Test complete workflows
- Use test database (SQLite in-memory)

### Property-Based Testing
- Use `proptest` for invariant testing
- Test edge cases automatically
- Example: `normalize_workspace` never panics

## Documentation Requirements

### Public API
- All public items must have doc comments
- Include examples in doc comments
- Document error conditions

### README Updates
- Update when adding new features
- Keep examples working
- Document breaking changes

### Changelog
- Update CHANGELOG.md with every release
- Follow [Keep a Changelog](https://keepachangelog.com/) format
- Include migration guides for breaking changes
