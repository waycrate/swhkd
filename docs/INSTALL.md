## Installation

# Dependencies:

## Runtime:

-   Policy Kit Daemon ( polkit )

## Compile time:

-   `rustup`
-   `make`

# Compiling:

-   `git clone https://github.com/shinyzenith/swhkd`
-   `make setup`
-   `make clean`
    -   `make` for a musl compile.
    -   `make glibc` for a glibc compile.
-   `sudo make install`

# Running:

`pkexec swhkd`
