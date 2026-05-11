# Engram Coding Conventions

## General Principles
- **Readability** over cleverness
- **Explicit** over implicit
- **Consistency** across the codebase

## Rust Conventions

### Naming
- Types: `PascalCase` (`Memory`, `SessionEndHandler`)
- Functions/methods: `snake_case` (`create_handler`, `handle_session_end`)
- Constants: `SCREAMING_SNAKE_CASE` (`SCHEMA_VERSION`, `MAX_CONTENT_LENGTH`)
- Modules: `snake_case` (`storage`, `search`, `hooks`)

### Error Handling
- Use `Result<T, EngramError>` for fallible operations
- Use `?` operator for error propagation
- Create specific error variants in `error.rs`

### Documentation
- Public API: `///` doc comments with examples
- Internal: `//` for single-line, `/* */` for multi-line
- Include `# Examples` section when practical

### Testing
- Unit tests in `#[cfg(test)] mod tests` at bottom of file
- Integration tests in `tests/` directory
- Use descriptive test names: `test_<function>_<scenario>`

## Python Conventions (SDK)

### Naming
- Classes: `PascalCase` (`EngramClient`)
- Functions/methods: `snake_case` (`create`, `list_memories`)
- Private: `_leading_underscore` (`_mcp_call`, `_build_params`)
- Avoid shadowing built-ins (use `filter_` not `filter`)

### Type Hints
- Use modern syntax: `dict[str, Any]` not `Dict[str, Any]`
- Always type public method parameters and return values
- Use `Optional[X]` or `X | None` consistently (Python 3.9+)

### Async
- Use `async with` for context managers
- All I/O methods must be `async def`
- Document async requirements in docstrings

## TypeScript Conventions (SDK)

### Naming
- Classes/Interfaces: `PascalCase` (`EngramClient`, `MemoryOptions`)
- Methods: `camelCase` (`createMemory`, `listMemories`)
- Parameters: `camelCase` (`memoryId`, `scopePath`)
- Convert to `snake_case` when calling API

### Types
- Prefer interfaces over types for objects
- Use `unknown` sparingly, create specific types
- Document all public methods with TSDoc (`/** */`)

### Async
- Use `async/await` consistently
- Return `Promise<T>` for async methods
- Handle errors with `try/catch`

## Git Conventions

### Commit Messages
Follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation changes
- `test:` test updates
- `refactor:` code refactoring

Example:
```
feat: add temporal graph contradictions detection

Implements memory_temporal_contradictions MCP tool
with workspace filtering support.
```

### Branches
- `main`: stable release branch
- `feature/description`: new features
- `fix/description`: bug fixes
- `docs/description`: documentation updates

## Code Review Checklist
- [ ] Code compiles without warnings (`cargo clippy`, `npm run lint`)
- [ ] Tests added/updated for changes
- [ ] Documentation updated (docstrings, README if needed)
- [ ] No sensitive data committed (API keys, passwords)
- [ ] Consistent with project conventions
- [ ] Error handling considered
