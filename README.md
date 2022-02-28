<p align=center>
  <img src="https://git.sr.ht/~shinyzenith/swhkd/blob/main/docs/assets/swhkd.png" alt=SWHKD width=60%>
  
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

## Installation

See [INSTALL.md](./docs/INSTALL.md) for installing swhkd.

Note: `swhks` is not a typo, it is the server process of the program.

## Running:
```bash
swhks &
pkexec swhkd
```
To refresh the config at runtime, make a script like so:

```bash
#!/bin/sh
sudo killall swhkd
pkexec swhkd
```

Mark it as executable using `chmod +x <path_to_refresh_script>`.

Then call it using `setsid -f <path_to_refresh_script>`. 

A better implementation using signals will be developed later.

## Configuration

Swhkd closely follows sxhkd syntax, so most existing sxhkd configs should be functional with swhkd.

The default configuration directory is `/etc/swhkd/swhkdrc`. If you don't like having to edit the file as root every single time, you can create a symlink from `~/.config/swhkd/swhkdrc` to `/etc/swhkd/swhkdrc`.

## Support server:

https://discord.gg/KKZRDYrRYW

## Contributors:

<a href="https://github.com/Shinyzenith/swhkd/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=waycrate/swhkd" />
</a>
