# 001: Git Backend Abstraction Layer

## Status
Accepted

## Context
repo-lens needs to interact with Git repositories but the choice of Git library affects performance, maintenance, and compatibility. Options include:

- **libgit2**: Mature C library with Rust bindings
- **gitoxide**: Pure Rust Git implementation
- **git2-rs**: Rust bindings for libgit2
- **subprocess**: Shell out to git binary

## Decision
Use a trait-based abstraction layer (`rl_git`) that allows swapping implementations. Start with stub implementation, evaluate real backends later.

## Consequences
- **Positive**: Flexibility to choose best backend for each use case
- **Positive**: Testability with mock backends
- **Negative**: Additional abstraction overhead
- **Neutral**: Implementation choice deferred until performance requirements known

## Implementation
```rust
pub trait GitBackend: Send + Sync {
    async fn open_repo(&self, path: &Path) -> Result<Box<dyn RepoHandle>>;
    // ... other methods
}
```
