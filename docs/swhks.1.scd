swhks(1) "github.com/shinyzenith/swhkd" "General Commands Manual"

# NAME

swhks - Server for swhkd, used to run user level commands over IPC.

# SYNOPSIS

*swhks*

# OPTIONS

*-h*, *--help*
	Print help message and quit.

*-V*, *--version*
	Print version information.

*-l*, *--log* <LOG_FILE_PATH>
	Set a log file path.
	If *XDG_DATA_HOME* exists then we use *swhks/swhks-current_time.log* relative to
	it, else we use *.local/share/swhks/swhks-current_time.log* relative to the
	user home directory.

*-d*, *--debug*
	Enable debug mode.

# AUTHORS

Maintained by Shinyzenith <aakashsensharma@gmail.com>, EdenQwQ <lsahlm1eden@gmail.com>, and Angelo Fallaria <ba.fallaria@gmail.com>.
For more information about development, see <https://github.com/waycrate/swhks>.

# SEE ALSO

	- *swhkd(1)*
	- *swhkd(5)*
	- *swhkd-keys(5)*
