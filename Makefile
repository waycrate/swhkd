# Destination dir, defaults to root. Should be overridden for packaging
# e.g. make DESTDIR="packaging_subdir" install
DESTDIR ?= "/"
DAEMON_BINARY := swhkd
SERVER_BINARY := swhks
BUILDFLAGS := --release
TARGET_DIR := /usr/bin
MAN1_DIR := /usr/share/man/man1
MAN5_DIR := /usr/share/man/man5
VERSION = $(shell awk -F ' = ' '$$1 ~ /version/ { gsub(/["]/, "", $$2); printf("%s",$$2) }' Cargo.toml)

ifneq ($(NO_RFKILL_SW_SUPPORT),)
	BUILDFLAGS += --features "no_rfkill"
endif

all: build

build:
	@cargo build $(BUILDFLAGS)

install:
	@find ./docs -type f -iname "*.1.gz" \
		-exec install -Dm 644 {} -t $(DESTDIR)/$(MAN1_DIR) \;
	@find ./docs -type f -iname "*.5.gz" \
		-exec install -Dm 644 {} -t $(DESTDIR)/$(MAN5_DIR) \;
	@install -Dm 755 ./target/release/$(DAEMON_BINARY) -t $(DESTDIR)/$(TARGET_DIR)
	@sudo chown root:root $(DESTDIR)/$(TARGET_DIR)/$(DAEMON_BINARY)
	@sudo chmod u+s $(DESTDIR)/$(TARGET_DIR)/$(DAEMON_BINARY)
	@install -Dm 755 ./target/release/$(SERVER_BINARY) -t $(DESTDIR)/$(TARGET_DIR)
	# Ideally, we would have a default config file instead of an empty one
	@if [ ! -f $(DESTDIR)/etc/$(DAEMON_BINARY)/$(DAEMON_BINARY)rc ]; then \
		touch ./$(DAEMON_BINARY)rc; \
		install -Dm 644 ./$(DAEMON_BINARY)rc -t $(DESTDIR)/etc/$(DAEMON_BINARY); \
	fi

uninstall:
	@$(RM) -f /usr/share/man/**/swhkd.*
	@$(RM) -f /usr/share/man/**/swhks.*
	@$(RM) $(TARGET_DIR)/$(SERVER_BINARY)
	@$(RM) $(TARGET_DIR)/$(DAEMON_BINARY)

check:
	@cargo fmt
	@cargo check
	@cargo clippy

release:
	@$(RM) -f Cargo.lock
	@$(MAKE) -s
	@zip -r "glibc-x86_64-$(VERSION).zip" ./target/release/swhkd ./target/release/swhks

test:
	@cargo test

clean:
	@cargo clean
	@$(RM) -f ./docs/*.gz
	@$(RM) -f $(DAEMON_BINARY)rc

setup:
	@rustup install stable
	@rustup default stable

.PHONY: check clean setup all install build release
