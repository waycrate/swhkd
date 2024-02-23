# AUR:

We have packaged `swhkd-git`. `swhkd-bin` has been packaged separately by a user of swhkd.

# Building:

`swhkd` and `swhks` install to `/usr/local/bin/` by default. You can change this behaviour by editing the [Makefile](../Makefile) variable, `DESTDIR`, which acts as a prefix for all installed files. You can also specify it in the make command line, e.g. to install everything in `subdir`: `make DESTDIR="subdir" install`.

# Dependencies:

**Runtime:**

-   Policy Kit Daemon ( polkit )
-   Uinput kernel module
-   Evdev kernel module

**Compile time:**

-   git
-   scdoc (If present, man-pages will be generated)
-   make
-   rustup

# Compiling:

-   `git clone https://github.com/waycrate/swhkd;cd swhkd`
-   `make setup`
-   `make clean`
    -   `make`
-   `sudo make install`

# Running:

```
swhks &
pkexec swhkd
```
