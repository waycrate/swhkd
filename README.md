<p align=center>
  <img src="https://git.sr.ht/~shinyzenith/swhkd/blob/main/assets/swhkd.png" alt=SWHKD width=60%>

  <p align="center">A next-generation hotkey daemon for Wayland/X11 written in <a href="https://www.rust-lang.org/">Rust</a>.</p>

  <p align="center">
  <a href="./LICENSE.md"><img src="https://img.shields.io/github/license/waycrate/swhkd?style=flat-square&logo=appveyor"></a>
  <img src="https://img.shields.io/badge/cargo-v1.2.1-green?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/issues/waycrate/swhkd?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/forks/waycrate/swhkd?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/stars/waycrate/swhkd?style=flat-square&logo=appveyor">
  </p>
</p>

## SWHKD

**S**imple **W**ayland **H**ot**K**ey **D**aemon

`swhkd` is a display protocol-independent hotkey daemon made in
[Rust](https://www.rust-lang.org). `swhkd` uses an easy-to-use configuration
system inspired by `sxhkd`, so you can easily add or remove hotkeys.

It also attempts to be a drop-in replacement for `sxhkd`, meaning your `sxhkd`
config file is also compatible with `swhkd`.

Because `swhkd` can be used anywhere, the same `swhkd` config can be used across
Xorg or Wayland desktops, and you can even use `swhkd` in a TTY.

## Installation and Building

[Installation and building instructions can be found here.](./INSTALL.md)

## Running

```bash
./swhks && doas ./swhkd
```

The doas or sudo can be skipped by making the swhkd binary a setuid binary.
This can be done by running the following command:

```bash
sudo chown root:root swhkd
sudo chmod u+s swhkd
```

then to start,
```bash
swhks &
swhkd
```

## Runtime signals

After opening `swhkd`, you can control the program through signals:

- `sudo pkill -USR1 swhkd` — Pause key checking
- `sudo pkill -USR2 swhkd` — Resume key checking
- `sudo pkill -HUP swhkd` — Reload config file

## Configuration

`swhkd` closely follows `sxhkd` syntax, so most existing `sxhkd` configs should
be functional with `swhkd`.
More information about the sxhkd syntax can be found in the official man pages from the [arch wiki](https://man.archlinux.org/man/sxhkd.1).

The default configuration file is in `~/.config/swhkd/swhkdrc` with a fallback to `etc/swhkd/swhkdrc`.

If you use Vim, you can get `swhkd` config syntax highlighting with the
[swhkd-vim](https://github.com/waycrate/swhkd-vim) plugin. Install it in
vim-plug with `Plug 'waycrate/swhkd-vim'`.

All supported key and modifier names are listed in `man 5 swhkd-keys`.

## Environment Variables

The environment variables are now sourced using the SWHKS binary, running in the background which are then supplemented
to the command that is to be run, thus emulating the environment variables in the default shell.

The commands are executed via *SHELL -c 'command'*, hence the environment is sourced from the default shell.
If the user wants to use a different set of environment variables, they can set the environment variables
in the default shell or export the environment variables within a logged in instance of their shell before
running the SWHKS binary.

## Autostart

### To autostart `swhkd` you can do one of two things

1. Add the commands from the ["Running"
   section](https://github.com/waycrate/swhkd#running) to your window managers
   configuration file.
1. Enable the [service
   file](https://github.com/waycrate/swhkd/tree/main/contrib/init) for your
   respective init system. Currently, only systemd and OpenRC service files
   exist and more will be added soon including Runit.

## Security

We use a server-client model to keep you safe. The daemon (`swhkd` — privileged
process) is responsible for listening to key events and running shell commands.
The server (`swhks` — non-privileged process) is responsible for keeping a track of the
environment variables and sending them to the daemon. The daemon
uses these environment variables while running the shell commands.
The daemon only runs shell commands that have been parsed from the config file and there is no way to
run arbitrary shell commands. The server is responsible for only sending the environment variables to the daemon and nothing else.
This separation of responsibilities ensures security.

So yes, you're safe!

## Support

1. https://matrix.to/#/#waycrate-tools:matrix.org
1. https://discord.gg/KKZRDYrRYW

## Contributors

<a href="https://github.com/Shinyzenith/swhkd/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=waycrate/swhkd" />
</a>

## Supporters:

1. [@CluelessTechnologist](https://github.com/CluelessTechnologist)
