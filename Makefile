all: build

build:
	@cargo build --release --target=x86_64-unknown-linux-musl

glibc:
	@cargo clean
	@cargo build --release

install:
	@mkdir -p /usr/local/bin
	@cp ./target/x86_64-unknown-linux-musl/release/swhkd /usr/local/bin/swhkd
	@chmod +x /usr/local/bin/swhkd

uninstall:
	@rm /usr/local/bin/swhkd

run:
	@cargo run --target=x86_64-unknown-linux-musl

check:
	@cargo fmt
	@cargo check --target=x86_64-unknown-linux-musl

clean:
	@cargo clean

setup:
	@rustup install stable
	@rustup default stable
	@rustup target add x86_64-unknown-linux-musl

.PHONY: check clean setup all run install build glibc
