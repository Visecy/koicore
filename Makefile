clean:
	cargo clean

deps:
	pip install maturin
	
ffi-test: clean
	. scripts/ffi-test.sh

py-test: clean
	. scripts/py-test.sh