# 002: API Contract as Single Source of Truth

## Status
Accepted

## Context
Need to ensure consistency between:
- Wire protocol (JSON serialization)
- Type definitions across crates
- Client/server expectations
- Documentation

## Decision
Define all API DTOs in a single `rl_api` crate that serves as the contract. All other crates depend on this for type definitions.

## Consequences
- **Positive**: Single source of truth prevents drift
- **Positive**: Type safety across component boundaries
- **Positive**: Automatic serialization consistency
- **Negative**: Changes require coordinated updates
- **Neutral**: No runtime contract validation needed

## Implementation
- All request/response types in `rl_api`
- Versioned API with `ApiVersion` enum
- Deterministic JSON serialization tests
