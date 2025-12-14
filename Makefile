all: build test

.PHONY: doc
doc:
	cargo doc --workspace --no-deps

.PHONY: build
build:
	cargo build --release --workspace

.PHONY: test
test:
	cargo test

.PHONY: ffi-test
ffi-test: build
	rm -f ./target/ffi-test/CMakeCache.txt
	cmake -B ./target/ffi-test ./crates/koicore_ffi/tests/cxx_api
	cmake --build ./target/ffi-test

.PHONY: coverage
coverage:
	cargo llvm-cov --lcov --output-path lcov.info

.PHONY: publish
publish:
	cargo publish || echo "Failed to publish root crate, continuing..."
	cargo publish --manifest-path crates/koicore_ffi/Cargo.toml || echo "Failed to publish koicore_ffi crate, continuing..."

.PHONY: clean
clean:
	cargo clean
