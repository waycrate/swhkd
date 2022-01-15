BUILDFLAGS := --release
TARGET_DIR := /usr/local/bin

all: build

build:
	@cargo build ${BUILDFLAGS} --target=x86_64-unknown-linux-musl
	@cp ./target/x86_64-unknown-linux-musl/release/swhkd ./bin/swhkd

glibc:
	@cargo clean
	@cargo build ${BUILDFLAGS}
	@cp ./target/release/swhkd ./bin/swhkd

install:
	@mkdir -p ${TARGET_DIR}
	@mv ./bin/swhkd ${TARGET_DIR}
	@chmod +x ${TARGET_DIR}/swhkd

uninstall:
	@rm ${TARGET_DIR}/swhkd

run:
	@cargo run --target=x86_64-unknown-linux-musl

check:
	@cargo test
	@cargo fmt
	@cargo check --target=x86_64-unknown-linux-musl

clean:
	@cargo clean

setup:
	@mkdir bin
	@rustup install stable
	@rustup default stable
	@rustup target add x86_64-unknown-linux-musl

.PHONY: check clean setup all run install build glibc
