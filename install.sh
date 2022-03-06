#!/bin/sh
rustup install stable
rustup default stable
rustup target add x86_64-unknown-linux-musl
pkexec cp ./swhkd.rules /etc/polkit-1/rules.d/swhkd.rules
