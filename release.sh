#!/bin/bash
version=$(awk -F = '/^version/ {print $2}' Cargo.toml | awk '{$1=$1;print}' | tr -d '"')
make
cd bin
zip -r "musl_libc-x86_64-$version.zip" swhkd swhks
cd ..
make glibc
cd bin
zip -r "glibc-x86_64-$version.zip" swhkd swhks
rm ./swhkd 
rm ./swhks
