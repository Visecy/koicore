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
ffi-test: cmake-build
	@echo "Running C++ API tests..."
	rm -f ./target/ffi-test/CMakeCache.txt
	cmake -B ./target/ffi-test ./crates/koicore_ffi/tests/cxx_api
	cmake --build ./target/ffi-test
	cd ./target/ffi-test && ctest --output-on-failure

.PHONY: cmake-build
cmake-build:
	@echo "Building koicore_ffi with CMake..."
	rm -rf ./target/cmake-build
	mkdir -p ./target/cmake-build
	cd ./target/cmake-build && cmake ../../crates/koicore_ffi -DCMAKE_BUILD_TYPE=Release
	cd ./target/cmake-build && cmake --build .

.PHONY: cmake-integration-test
cmake-integration-test:
	@echo "Running CMake integration tests..."
	rm -rf ./target/cmake-integration-test
	mkdir -p ./target/cmake-integration-test
	cd ./target/cmake-integration-test && cmake ../../crates/koicore_ffi/tests/cmake_integration -D USE_LOCAL_KOICORE_FFI=ON
	cd ./target/cmake-integration-test && cmake --build .
	cd ./target/cmake-integration-test && ctest --output-on-failure

.PHONY: cmake-install-test
cmake-install-test: cmake-build
	@echo "Testing CMake installation..."
	rm -rf ./target/cmake-install
	mkdir -p ./target/cmake-install
	cd ./target/cmake-build && cmake --install . --prefix ../../target/cmake-install
	@echo "Installation test completed. Check ./target/cmake-install for installed files."

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
	rm -rf ./target/ffi-test
	rm -rf ./target/cmake-build
	rm -rf ./target/cmake-integration-test
	rm -rf ./target/cmake-install
