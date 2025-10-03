.PHONY: build
build:
	cargo build

.PHONY: test
test:
	cargo test
	cargo run --example echo

.PHONY: publish
publish:
	cargo publish

.PHONY: lint
lint:
	cargo clippy -- --deny warnings
	cargo fmt --check

.PHONY: format
format:
	cargo clippy --fix --allow-no-vcs
	cargo fmt
