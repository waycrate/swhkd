DAEMON_BINARY := swhkd
SERVER_BINARY := swhks
BUILDFLAGS := --release
POLKIT_DIR := /usr/share/polkit-1/actions
POLKIT_POLICY_FILE := com.github.swhkd.pkexec.policy
# Remember to edit the TARGET_DIR in policy file too if you do change it.
TARGET_DIR := /usr/bin
VERSION=$(shell awk -F ' = ' '$$1 ~ /version/ { gsub(/["]/, "", $$2); printf("%s",$$2) }' Cargo.toml)

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
	@cp ./$(POLKIT_POLICY_FILE) $(POLKIT_DIR)/$(POLKIT_POLICY_FILE)
	@chmod +x $(TARGET_DIR)/$(DAEMON_BINARY)
	@chmod +x $(TARGET_DIR)/$(SERVER_BINARY)

uninstall:
	@rm $(TARGET_DIR)/$(SERVER_BINARY)
	@rm $(TARGET_DIR)/$(DAEMON_BINARY)
	@rm $(POLKIT_DIR)/$(POLKIT_POLICY_FILE)

check:
	@cargo fmt
	@cargo check --target=x86_64-unknown-linux-musl
	@cargo clippy

release:
	@rm Cargo.lock
	@$(MAKE) -s
	@cd bin; zip -r "musl_libc-x86_64-$(VERSION).zip" swhkd swhks
	@$(MAKE) -s glibc
	@cd bin; zip -r "glibc-x86_64-$(VERSION).zip" swhkd swhks; rm ./swhkd; rm ./swhks

clean:
	@cargo clean

setup:
	@mkdir -p ./bin
	@rustup install stable
	@rustup default stable
	@rustup target add x86_64-unknown-linux-musl

.PHONY: check clean setup all install build glibc release
