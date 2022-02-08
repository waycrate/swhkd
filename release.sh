#!/bin/bash
version=$(awk -F = '/version/ {print $2}' Cargo.toml | awk '{$1=$1;print}' | tr -d '"')
make
zip -r "musl_libc-x86_64-$version.zip" ./bin/swhkd ./bin/swhks
make glibc
zip -r "glibc-x86_64-$version.zip" ./bin/swhkd ./bin/swhks
