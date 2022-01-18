DAEMON_BINARY := swhkd
CLIENT_BINARY := swhkc
BUILDFLAGS := --release
POLKIT_DIR := /etc/polkit-1/rules.d
POLKIT_RULE := swhkd.rules
TARGET_DIR := /usr/local/bin
DEPENDENCIES = rustup

all: build

CHECK := $(foreach exec,$(DEPENDENCIES),\
	$(if $(shell which $(exec)),some string,$(error "No $(exec) in PATH")))

build:
	@cargo build $(BUILDFLAGS) --target=x86_64-unknown-linux-musl
	@cp ./target/x86_64-unknown-linux-musl/release/$(DAEMON_BINARY) ./bin/$(DAEMON_BINARY)
	@cp ./target/x86_64-unknown-linux-musl/release/$(CLIENT_BINARY) ./bin/$(CLIENT_BINARY)

glibc:
	@cargo build $(BUILDFLAGS)
	@cp ./target/release/$(DAEMON_BINARY) ./bin/$(DAEMON_BINARY)
	@cp ./target/release/$(CLIENT_BINARY) ./bin/$(CLIENT_BINARY)

install:
	@mkdir -p $(TARGET_DIR)
	@mkdir -p $(POLKIT_DIR)
	@mkdir -p /etc/$(DAEMON_BINARY)
	@touch /etc/$(DAEMON_BINARY)/$(DAEMON_BINARY)rc
	@cp ./bin/$(DAEMON_BINARY) $(TARGET_DIR)
	@cp ./bin/$(CLIENT_BINARY) $(TARGET_DIR)
	@cp ./$(POLKIT_RULE) $(POLKIT_DIR)/$(POLKIT_RULE)
	@chmod +x $(TARGET_DIR)/$(DAEMON_BINARY)

uninstall:
	@rm $(TARGET_DIR)/$(CLIENT_BINARY)
	@rm $(TARGET_DIR)/$(DAEMON_BINARY)
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
