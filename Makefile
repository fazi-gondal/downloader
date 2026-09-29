.PHONY: test lint fmt check

# Unit tests (pure logic — FormatBuilder, DownloadManager, models)
test:
	cargo test --lib -- --nocapture

# Clippy (requires network + successful dependency fetch)
lint:
	cargo clippy --all-targets -- -D warnings

# Format
fmt:
	cargo fmt --all

# Combined quality gate
check: fmt lint test
