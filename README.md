# repo-lens

High-performance Git UI backend + CLI built in Rust. Core provides cancelable, incremental, testable Git queries via a stable API contract.

## Overview

repo-lens is a backend-first Git UI system designed for performance and correctness. It provides:

- **Stable API contract** for UI consumption
- **Incremental caching** to avoid repo-wide recomputation
- **Streaming responses** for heavy results
- **Cancelable queries** that respect user intent
- **Mechanical testing** against canonical git behavior

## Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   rl_cli    │    │   rl_ipc    │    │   rl_core   │
│  (binary)   │◄──►│  (library)  │◄──►│  (library)  │
└─────────────┘    └─────────────┘    └─────────────┘
       ▲                   ▲                   ▲
       │                   │                   │
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   rl_api    │    │   rl_git    │    │  rl_index   │
│  (library)  │    │  (library)  │    │  (library)  │
└─────────────┘    └─────────────┘    └─────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.70+ with 2021 edition
- Git (for testing)

### Build

```bash
# Clone the repository
git clone https://github.com/your-org/repo-lens.git
cd repo-lens

# Build all crates
cargo build --workspace

# Build optimized release
cargo build --workspace --release
```

### Run

```bash
# Show repository status (JSON output)
./target/debug/repo-lens status --repo /path/to/git/repo

# Show repository status (pretty output)
./target/debug/repo-lens --pretty status --repo /path/to/git/repo

# Get commit log
./target/debug/repo-lens log --repo /path/to/git/repo

# Run benchmarks
./target/debug/repo-lens-bench
```

## Development

### Testing

```bash
# Run all tests
cargo test --workspace

# Run with coverage (requires llvm-tools)
cargo llvm-cov --workspace --lcov --output-path lcov.info

# Run specific crate tests
cargo test -p rl_api
```

### Benchmarks

```bash
# Run performance benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench --bench status_bench
```

### Code Quality

```bash
# Format code
cargo fmt --workspace

# Lint code
cargo clippy --workspace -- -D warnings

# Check compilation
cargo check --workspace
```

### Documentation

```bash
# Build docs
cargo doc --workspace --open

# Serve docs
cargo doc --workspace && python3 -m http.server 8000 -d target/doc
```

## API Contract

The API uses JSON-RPC style communication with versioned DTOs:

```json
{
  "version": "v0",
  "id": "request-123",
  "payload": {
    "Status": {
      "repo_path": "/path/to/repo"
    }
  }
}
```

See [docs/contracts/](docs/contracts/) for detailed API documentation.

## Performance Goals

| Operation | Target P95 | Status |
|-----------|------------|--------|
| Status | 50ms | ✅ Stub |
| Log (page) | 80ms | ✅ Stub |
| Diff Summary | 120ms | ✅ Stub |

## Contributing

1. **Fork** the repository
2. **Create** a feature branch
3. **Implement** your changes
4. **Test** thoroughly: `cargo test --workspace`
5. **Format**: `cargo fmt --workspace`
6. **Lint**: `cargo clippy --workspace`
7. **Submit** a pull request

### Development Workflow

- **Backend-first**: Implement Git logic in `rl_git` and `rl_core`
- **API stability**: Changes to `rl_api` require version bumps
- **Performance**: All changes must maintain performance budgets
- **Testing**: Golden tests against canonical `git` behavior

## Project Structure

```
repo-lens/
├── crates/
│   ├── rl_api/          # API DTOs and contracts
│   ├── rl_git/          # Git backend abstraction
│   ├── rl_index/        # Caching and indices
│   ├── rl_core/         # Query engine and scheduling
│   ├── rl_ipc/          # Transport layer
│   ├── rl_cli/          # Command-line interface
│   ├── rl_bench/        # Performance benchmarks
│   └── rl_fixtures/     # Test data generators
├── docs/
│   ├── contracts/       # API documentation
│   └── decisions/       # Architectural decisions
├── .github/
│   └── workflows/       # CI/CD pipelines
└── system_map.json      # Project specification
```

## License

Licensed under MIT OR Apache-2.0.

## Roadmap

See [system_map.json](system_map.json) for detailed milestones:

- **M0**: Scaffolding ✅
- **M1**: Status + Watch
- **M2**: Log + Show + Graph
- **M3**: Diff + Performance baselines
