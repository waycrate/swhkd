## systemd/Sway Instructions

To have systemd automatically start `swhkd` for you when you log in:

1. Copy `hotkeys.sh` into your preferred directory
2. `chmod +x hotkeys.sh`
3. Copy `hotkeys.service` and `sway-session.target` into your `$XDG_CONFIG_DIR/systemd/user`
4. Using a text editor, uncomment line 7 of `hotkeys.service` and change the path accordingly
5. Add the following line to your sway config file:

      `exec_always "systemctl --user import-environment; systemctl --user start sway-session.target"`
6. In a terminal: `systemctl --user enable hotkeys.service`

You can use services like this to start other Sway-specific daemons, as well. For more information, see https://wiki.archlinux.org/title/Sway#Manage_Sway-specific_daemons_with_systemd
