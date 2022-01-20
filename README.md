<p align=center>
  <img src="./assets/swhkd.png" alt=SWHKD width=60%>
  
  <p align=center>A next-generation hotkey daemon for Wayland/X11 written in Rust.</p>
  
  <p align="center">
  <a href="./LICENSE.md"><img src="https://img.shields.io/github/license/shinyzenith/swhkd?style=flat-square&logo=appveyor"></a>
  <img src="https://img.shields.io/badge/cargo-v0.1.0-green?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/issues/shinyzenith/swhkd?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/forks/shinyzenith/swhkd?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/stars/shinyzenith/swhkd?style=flat-square&logo=appveyor">
  </p>
</p>

## SWHKD

swhkd is a display protocol-independent hotkey daemon made in Rust. swhkd uses an easy-to-use configuration system inspired by sxhkd so you can easily add or remove hotkeys.

It is also a drop-in replacement for sxhkd, meaning, your sxhkd config file is also compatible with swhkd.

Because swhkd can be used anywhere, the same swhkd config can be used across Xorg or Wayland desktops, and you can even use swhkd in a tty.

**Note: The project isn't complete yet.**
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

# Support server:

https://discord.gg/KKZRDYrRYW

# Contributors:

<a href="https://github.com/Shinyzenith/swhkd/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Shinyzenith/swhkd" />
</a>
