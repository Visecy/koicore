.PHONY: build
build:
	cargo build --release --workspace

.PHONY: test
test:
	cargo test

ffi-test: clean
	. scripts/ffi-test.sh

.PHONY: clean
clean:
	cargo clean
