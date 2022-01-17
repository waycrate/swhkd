BINARY := swhkd
BUILDFLAGS := --release
POLKIT_DIR := /etc/polkit-1/rules.d
POLKIT_RULE := swhkd.rules
TARGET_DIR := /usr/local/bin

all: build

build:
	@cargo build $(BUILDFLAGS) --target=x86_64-unknown-linux-musl
	@cp ./target/x86_64-unknown-linux-musl/release/$(BINARY) ./bin/$(BINARY)

glibc:
	@cargo build $(BUILDFLAGS)
	@cp ./target/release/$(BINARY) ./bin/$(BINARY)

install:
	@mkdir -p $(TARGET_DIR)
	@mkdir -p $(POLKIT_DIR)
	@mkdir -p /etc/$(BINARY)
	@touch /etc/$(BINARY)/$(BINARY)rc
	@cp ./bin/$(BINARY) $(TARGET_DIR)
	@cp ./$(POLKIT_RULE) $(POLKIT_DIR)/$(POLKIT_RULE)
	@chmod +x $(TARGET_DIR)/$(BINARY)

uninstall:
	@rm $(TARGET_DIR)/$(BINARY)
	@rm $(POLKIT_DIR)/$(POLKIT_RULE)

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
