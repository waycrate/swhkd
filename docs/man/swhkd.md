# SWHKD

**S**imple **W**ayland **H**ot**K**ey **D**aemon

swhkd is a display protocol-independent hotkey daemon made in Rust. swhkd uses an easy-to-use configuration system inspired by sxhkd so you can easily add or remove hotkeys.

It also attempts to be a drop-in replacement for sxhkd, meaning, your sxhkd config file is also compatible with swhkd.

Because swhkd can be used anywhere, the same swhkd config can be used across Xorg or Wayland desktops, and you can even use swhkd in a tty.

**Note: The project isn't complete yet.**

# RUNNING

```
swhks &
pkexec swhkd
```

**Note: swhks is NOT a typo. Please check the man page [ swhks(1) ] for more info.**

# SEE ALSO

swhks(1)
