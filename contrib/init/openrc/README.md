## OpenRC Instructions

To have OpenRC automatically start `swhkd` for you:

1. `chmod +x swhkd`
2. Copy `swhkd` into /etc/init.d/
3. Run `sudo rc-update add swhkd`
4. Run `swhks` on login ( Add it to your `.xinitrc` file or your setup script )

