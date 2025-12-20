CARGO ?= cargo
PROFILE ?= debug

.PHONY: help fmt fmt-check clippy test test-fast bench bench-quick bench-baseline-save bench-baseline-compare build build-release clean ci

help:
	@echo "Available targets:"
	@echo "  help         Show this help message"
	@echo "  fmt          Format code with rustfmt"
	@echo "  fmt-check    Check code formatting without modifying"
	@echo "  clippy       Run clippy with warnings as errors"
	@echo "  test         Run all tests in the workspace"
	@echo "  test-fast    Run tests excluding ignored/slow tests"
	@echo "  bench        Run all benchmarks in the workspace"
	@echo "  bench-quick  Run lightweight benchmark harness"
	@echo "  bench-baseline-save    Save current run as baseline"
	@echo "  bench-baseline-compare Compare current run against baseline"
	@echo "  build        Build workspace in debug mode"
	@echo "  build-release Build workspace in release mode"
	@echo "  clean        Clean build artifacts"
	@echo "  ci           Run CI checks (fmt-check, clippy, test)"

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

test:
	$(CARGO) test --workspace

test-fast:
	$(CARGO) test --workspace

bench:
	$(CARGO) bench --workspace

bench-quick:
	$(CARGO) run -p rl_bench -- run

bench-baseline-save:
	$(CARGO) run -p rl_bench -- baseline save

bench-baseline-compare:
	$(CARGO) run -p rl_bench -- baseline compare crates/rl_bench/baselines/local.json

build:
	$(CARGO) build --workspace --profile $(PROFILE)

build-release:
	$(CARGO) build --workspace --release

clean:
	$(CARGO) clean

ci: fmt-check clippy test
