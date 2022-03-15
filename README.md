<p align=center>
  <img src="https://git.sr.ht/~shinyzenith/swhkd/blob/main/docs/assets/swhkd.png" alt=SWHKD width=60%>
  
  <p align=center>A next-generation hotkey daemon for Wayland/X11 written in Rust.</p>
  
  <p align="center">
  <a href="./LICENSE.md"><img src="https://img.shields.io/github/license/waycrate/swhkd?style=flat-square&logo=appveyor"></a>
  <img src="https://img.shields.io/badge/cargo-v1.1.5-green?style=flat-square&logo=appveyor">
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

## Runtime signals

After opening swhkd, you can control the program through signals:

- `sudo pkill -USR1 swhkd` - Pause key checking
- `sudo pkill -USR2 swhkd` - Resume key checking
- `sudo pkill -HUP swhkd` - Reload config file

## Configuration

Swhkd closely follows sxhkd syntax, so most existing sxhkd configs should be functional with swhkd.

The default configuration directory is `/etc/swhkd/swhkdrc`. If you don't like having to edit the file as root every single time, you can create a symlink from `~/.config/swhkd/swhkdrc` to `/etc/swhkd/swhkdrc`.

If you use Vim, you can get swhkd config syntax highlighting with the
[swhkd-vim](https://github.com/waycrate/swhkd-vim) plugin. Install it in
vim-plug with `Plug 'waycrate/swhkd-vim'`.

## Security
We use a server-client model to keep you safe. The daemon ( swhkd - privileged process ) communicates to the server ( swhks - running as non root user ) after checking for valid keybinds. Since the daemon is totally separate from the server, no other process can read your keystrokes. As for shell commands, you might be thinking that any program can send shell commands to the server and that's true! But the server runs the commands as the currently logged in user so no extra permissions are provided ( This is essentially the same as any app on your desktop calling shell commands ).

So yes, you're safe!

## Support:

1) https://matrix.to/#/#waycrate-tools:matrix.org
2) https://discord.gg/KKZRDYrRYW

## Contributors:

<a href="https://github.com/Shinyzenith/swhkd/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=waycrate/swhkd" />
</a>
