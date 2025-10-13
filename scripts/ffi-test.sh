LIB_NAME=koicore_ffi
CRATE_DIR=crates/${LIB_NAME}
TARGET_DIR=target/release
TARGET_NAME=ffi-test

cd ${CRATE_DIR}
cargo build --release

cd ../../${TARGET_DIR}
gcc -o ${TARGET_NAME} ../../${CRATE_DIR}/test.c -I../../${CRATE_DIR} -L. -l${LIB_NAME}
LD_LIBRARY_PATH=. ./${TARGET_NAME}