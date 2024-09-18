# AUR:

We have packaged `swhkd-git`. `swhkd-bin` has been packaged separately by a user of swhkd.

## NixOS

For now flake users only.

This repo contains a NixOS Module for swhkd service.
To enable module add an input first and import to modules:
```nix
{
  inputs = {
    swhkd.url = "github:waycrate/swhkd";
  }
  outputs = {nixpkgs, swhkd, ...} @ inputs: {
    nixosConfigurations.HOSTNAME = nixpkgs.lib.nixosSystem {
      specialArgs = { inherit inputs; };
      modules = [
        ./configuration.nix
        swhkd.nixosModules.default
      ];
    };
  } 
}
```
After importing you should be able to use it in your configuration.nix file:
```nix
{ inputs
, ...
}:
{
  services.swhkd = {
    enable = true;
    package = inputs.swhkd.packages.${system}.default;
    cooldown = 300;
    settings = ''
super + return
  alacritty
    '';
  };
}
```
* Do not forget to start/add to autostart swhkd of your system after login.
* Replace HOSTNAME with your oun

ps. this module will be updated after Security Model improve, but it is already good enough to use

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
