swhkd(5) "github.com/waycrate/swhkd" "File Formats Manual"

# NAME

swhkd	- Hotkey daemon inspired by sxhkd written in Rust

# CONFIG FILE

- A global config can be defined in *~/.config/swhkd/swhkdrc*, with a
fallback to  */etc/swhkd/swhkdrc*. Swhkd attempts to look in your *$XDG_CONFIG_HOME*, failing which it defaults to *~/.config*.
- A local config overrides the global one. Local configs should be placed in the root of the project.
- The config file can also be specified with the *-c* flag.

# ENVIRONMENT

- The environment variables are now sourced using the SWHKS binary, running in the background.
- The environment variables are then supplemented to the command that is to be run, thus emulating the
environment variables in the default shell.
- The commands are executed via *SHELL -c 'command'*, hence the environment is sourced from the default shell.
- If the user wants to use a different set of environment variables, they can set the environment variables
in the default shell or export the environment variables within a logged in instance of their shell before
running the SWHKS binary.

# SYNTAX

The syntax of the configuration file is identical to sxhkd and builds upon it.
More information about the syntax can be found from the official sxhkd documentation:
https://man.archlinux.org/man/sxhkd.1, however a brief summary of it is provided below.

Each line of the configuration file is interpreted as so:
	- If it is empty or starts with #, it is ignored.
	- If it starts with a space, it is read as a command.
	- Otherwise, it is read as a hotkey.

For valid keys and modifiers, check *swhkd-keys(5)*

# EXAMPLE

```
# Import another configuration file.
# NOTE: the path provided must be absolute and not relative such as `~`.
include /home/YourUserName/.config/swhkd/swhkdrc

ignore alt + print # globally ignore a key binding

# terminal
super + ReTuRn # case insensitive
	alacritty

super + shift + enter # enter = return
	kitty

# file manager
super + shift + f
	pcmanfm

# web-browser
super + w
	firefox

# bspwm
super + {_,shift + }{h,j,k,l}
	bspc node -{f,s} {west,south,north,east}

super + ctrl + alt + {Left\
,Down\
,Up\
,Right}
  n=10; \
  { d1=left;   d2=right;  dx=-$n; dy=0;   \
  , d1=bottom; d2=top;    dx=0;   dy=$n;  \
  , d1=top;    d2=bottom; dx=0;   dy=-$n; \
  , d1=right;  d2=left;   dx=$n;  dy=0;   \
  } \
  bspc node --resize $d1 $dx $dy || bspc node --resize $d2 $dx $dy

super + {\,, .}
  bspc node -f {next.local,prev.local}

# screenshot
print
	scrot

any + print # any represent at least one of the valid modifiers
	scrot -s

# Append with @ to run on key-release.
@super + shift + f
	pcmanfm

# Append with ~ to emit the hotkey after the command is triggered. Aka, don't swallow the hotkey.
~super + shift + f
	pcmanfm

super + m
	# commands starting with @ are internal commands.
	# internal commands can be combined with normal commands with '&&'.
	# '@enter' pushes a mode into the mode stack and starts listening only the
	# key bindings defined in that mode
	@enter music && echo "music" > ~/.config/waybar/swhkd-mode

mode music # use the mode statement to define a mode
q
	# '@escape' pops the current mode out of the mode stack
	# the default mode is 'normal mode', which is always on the bottom of the mode
	# stack and can never be escaped
	@escape && echo "normal" > ~/.config/waybar/swhkd-mode
{n, p, space, r, z, y}
	mpc {next, prev, toggle, repeat, random, single}
endmode # use endmode if you want to set more key bindings for normal mode

# mode options are declared after the mode name
# swallow: don't emit any event through uinput
# oneoff: automatically escape a mode when a keybinding defined in it is evoked
mode option_demo swallow oneoff
a
	echo 0
b
	@escape # escaping in a 'oneoff' mode pops two modes out of the mode stack.
endmode

```
# AUTHORS

Maintained by Shinyzenith <aakashsensharma@gmail.com>, EdenQwQ <lsahlm1eden@gmail.com>, and Angelo Fallaria <ba.fallaria@gmail.com>.
For more information about development, see <https://github.com/waycrate/swhkd>.

# SEE ALSO

	- *swhkd(1)*
	- *swhkd-keys(5)*
	- *swhks(1)*
