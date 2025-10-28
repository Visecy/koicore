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

.PHONY: clean
clean:
	cargo clean
