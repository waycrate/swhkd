<p align=center>
  <img src="./docs/assets/swhkd.png" alt=SWHKD width=60%>
  
  <p align=center>A next-generation hotkey daemon for Wayland/X11 written in Rust.</p>
  
  <p align="center">
  <a href="./LICENSE.md"><img src="https://img.shields.io/github/license/waycrate/swhkd?style=flat-square&logo=appveyor"></a>
  <img src="https://img.shields.io/badge/cargo-v1.0.0-green?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/issues/waycrate/swhkd?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/forks/waycrate/swhkd?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/stars/waycrate/swhkd?style=flat-square&logo=appveyor">
  </p>
</p>

## SWHKD

**S**imple **W**ayland **H**ot**K**ey **D**aemon

swhkd is a display protocol-independent hotkey daemon made in Rust. swhkd uses an easy-to-use configuration system inspired by sxhkd so you can easily add or remove hotkeys.

It also attempts to be a drop-in replacement for sxhkd, meaning, your sxhkd config file is also compatible with swhkd.

Because swhkd can be used anywhere, the same swhkd config can be used across Xorg or Wayland desktops, and you can even use swhkd in a tty.

**Note: The project is a WIP.**
**BUT!! It does work right now however it's not a drop-in replacement yet. [Example config file](./docs/swhkdrc).**

## Installation

See [INSTALL.md](./docs/INSTALL.md) for installing swhkd.

## Running:
```bash
swhks &
pkexec swhkd
```
To refresh the config at runtime, make a script like so:

```bash
#!/bin/sh
pkexec killall swhkd
pkexec swhkd
```

Then call it using `setsid -f sh <path_to_refresh_script>`. 

A better implementation using signals will be developed later.

## Support server:

https://discord.gg/KKZRDYrRYW

## Contributors:

<a href="https://github.com/Shinyzenith/swhkd/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=waycrate/swhkd" />
</a>
