swhkd(1) "github.com/shinyzenith/swhkd" "General Commands Manual"

# NAME

swhkd - Hotkey daemon inspired by sxhkd written in rust

# SYNOPSIS

*pkexec swhkd* [_flags_]

# CONFIG FILE

The config file goes in */etc/swhkd/swhkdrc*. Since swhkd is written with a pkexec privilege escalation model in mind, we can't detect
*$XDG_CONFIG_HOME*.

This can be avoided as such:
	- Using the *include* statement in your config file.
	- Using the *-c* flag to mention a custom config file at runtime.
	- Symlinking the config file from *~/.config/swhkd/swhkdrc* or any other directory of choice to */etc/swhkd/swhkdrc*.

More about the config file syntax in `swhkd(5)`

# OPTIONS

*-h*, *--help*
	Print help message and quit.

*-V*, *--version*
	Print version information.

*-c*, *--config* <CONFIG_FILE_PATH>
	Set a custom config file path.

*-C*, *--cooldown* <COOLDOWN_IN_MS>
	Set a custom repeat cooldown duration. Default is 250ms. Most wayland
	compositors handle this server side however, either way works.

*-d*, *--debug*
	Enable debug mode.

*-D, --device* <DEVICE_NAME>
	Manually set the keyboard devices to use. Can occur multiple times.

# SIGNALS

	- Reload config file: `sudo pkill -HUP swhkd`
	- Pause Hotkey checking: `sudo pkill -USR1 swhkd`
	- Resume key checking: `sudo pkill -USR2 swhkd`

# AUTHORS

Maintained by Shinyzenith <aakashsensharma@gmail.com>, EdenQwQ <lsahlm1eden@gmail.com>, and Angelo Fallaria <ba.fallaria@gmail.com>.
For more information about development, see <https://github.com/waycrate/swhkd>.

# SEE ALSO

	- *swhkd(5)*
	- *swhkd-keys(5)*
	- *swhks(1)*
