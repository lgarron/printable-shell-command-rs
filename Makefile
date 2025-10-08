.PHONY: build
build:
	cargo build

.PHONY: test
test:
	cargo test
	cargo run --example echo
	cargo run --example ffmpeg
	cargo run --example cargo-version

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

.PHONY: clean
clean:
	# No-op in this repo

.PHONY: reset
reset: clean
	rm -rf ./target
