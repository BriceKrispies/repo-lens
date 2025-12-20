# 003: Incremental Caching Strategy

## Status
Accepted

## Context
Git operations can be expensive, especially on large repositories. Need to balance:
- Memory usage (can't cache everything)
- Speed (avoid recomputation)
- Correctness (cache invalidation)

## Decision
Use bounded, incremental caches with LRU eviction. Separate caches for:
- Commit graph windows
- Tree snapshots
- Diff chunks
- Blame data

## Consequences
- **Positive**: Fast subsequent operations
- **Positive**: Memory-bounded resource usage
- **Negative**: Cache misses on first access
- **Negative**: Complexity of invalidation logic

## Implementation
- `IndexManager` coordinates all caches
- Configurable memory limits
- Event-driven invalidation (repo changes)
