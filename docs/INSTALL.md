# AUR:
`swhkd-git` `swhkd-musl-git` have been packaged. `swhkd-bin` & `swhkd-musl-bin` will be released soon.

# Cargo

`cargo install --locked --git https://github.com/waycrate/swhkd`

# Install

`swhkd` and `swhks` install to `/usr/local/bin/` by default. You can change this behaviour by editing the [Makefile](../Makefile) variable, `TARGET_DIR`.

# Dependencies:

## Runtime:

-   Policy Kit Daemon ( polkit )

## Compile time:

-   rustup
-   make

# Compiling:

-   `git clone https://github.com/waycrate/swhkd;cd swhkd`
-   `make setup`
-   `make clean`
    -   `make` for a musl compile.
    -   `make glibc` for a glibc compile.
-   `sudo make install`

# Running:
`swhks`
`pkexec swhkd`
