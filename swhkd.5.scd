swhkd(5) "github.com/waycrate/swhkd" "File Formats Manual"

# NAME

swhkd	- Hotkey daemon inspired by sxhkd written in rust

# CONFIG FILE

	- A global config can be defined in */etc/swhkd/swhkdrc*. Swhkd attempts to look in your *$XDG_CONFIG_HOME*, failing which it defaults to *~/.config*.
	- A local config overrides the global one. Local configs should be placed in the root of the project.

# SYNTAX

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

```
# AUTHORS

Maintained by Shinyzenith <aakashsensharma@gmail.com>, EdenQwQ <lsahlm1eden@gmail.com>, and Angelo Fallaria <ba.fallaria@gmail.com>.
For more information about development, see <https://github.com/waycrate/swhkd>.

# SEE ALSO

	- *swhkd(1)*
	- *swhkd-keys(5)*
	- *swhks(1)*
