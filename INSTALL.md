# AUR:

We have packaged `swhkd-git`. `swhkd-bin` has been packaged separately by a user of swhkd.

# NixOS:

Basic installation and autorun on NixOS
```nix
# Add inputs to your flake
  inputs.swhkd.url = "github:waycrate/swhkd";
  inputs.swhkd.inputs.nixpkgs.follows = "nixpkgs";
...
# Add package to your configuration
  environment.systemPackages = [ inputs.swhkd.packages.${pkgs.hostPlatform.system}.default ];
...
# Enable polkit and create rule
  security.polkit.enable = true;
  security.polkit.extraConfig = ''
    polkit.addRule(function(action, subject) {
        if (action.id == "com.github.swhkd.pkexec"  &&
            subject.local == true &&
            subject.active == true &&) {
                return polkit.Result.YES;
        }
    });
  '';
  ...
# Autorun daemon with systemd
  systemd.user.services.swhkd = {
    description = "swhkd hotkey daemon";
    bindsTo = ["default.target"];
    script = ''
      /run/wrappers/bin/pkexec ${inputs.swhkd.packages.${pkgs.hostPlatform.system}.default}/bin/swhkd \
        --config $XDG_CONFIG_HOME/swhkd/swhkdrc \
        --cooldown 250
      '';
    serviceConfig.Restart = "always";
    wantedBy = ["default.target"];
  };
```
After that add 'swhks &' to autorun of your desktop enviroment or window manager

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
-   libudev (in Debian, the package name is `libudev-dev`)
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
