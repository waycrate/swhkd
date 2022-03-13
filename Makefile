DAEMON_BINARY := swhkd
SERVER_BINARY := swhks
BUILDFLAGS := --release
POLKIT_DIR := /etc/polkit-1/rules.d
POLKIT_RULE := 00-swhkd.rules
TARGET_DIR := /usr/bin

all: build

build:
	@cargo build $(BUILDFLAGS) --target=x86_64-unknown-linux-musl
	@cp ./target/x86_64-unknown-linux-musl/release/$(DAEMON_BINARY) ./bin/$(DAEMON_BINARY)
	@cp ./target/x86_64-unknown-linux-musl/release/$(SERVER_BINARY) ./bin/$(SERVER_BINARY)

glibc:
	@cargo build $(BUILDFLAGS)
	@cp ./target/release/$(DAEMON_BINARY) ./bin/$(DAEMON_BINARY)
	@cp ./target/release/$(SERVER_BINARY) ./bin/$(SERVER_BINARY)

install:
	@mkdir -p $(TARGET_DIR)
	@mkdir -p $(POLKIT_DIR)
	@mkdir -p /etc/$(DAEMON_BINARY)
	@touch /etc/$(DAEMON_BINARY)/$(DAEMON_BINARY)rc
	@cp ./bin/$(DAEMON_BINARY) $(TARGET_DIR)
	@cp ./bin/$(SERVER_BINARY) $(TARGET_DIR)
	@cp ./$(POLKIT_RULE) $(POLKIT_DIR)/$(POLKIT_RULE)
	@chmod +x $(TARGET_DIR)/$(DAEMON_BINARY)
	@chmod +x $(TARGET_DIR)/$(SERVER_BINARY)

uninstall:
	@rm $(TARGET_DIR)/$(SERVER_BINARY)
	@rm $(TARGET_DIR)/$(DAEMON_BINARY)
	@rm $(POLKIT_DIR)/$(POLKIT_RULE)

check:
	@cargo fmt
	@cargo check --target=x86_64-unknown-linux-musl

clean:
	@cargo clean

setup:
	@mkdir -p ./bin
	@rustup install stable
	@rustup default stable
	@rustup target add x86_64-unknown-linux-musl

.PHONY: check clean setup all install build glibc
