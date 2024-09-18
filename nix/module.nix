self: 
{ config
, lib
, pkgs
, ...
}:
let
  cfg = config.services.swhkd;
  inherit (pkgs.stdenv.hostPlatform) system;

  inherit (lib) types;
  inherit (lib.modules) mkIf;
  inherit (lib.options) mkOption mkEnableOption;
in
{
  options.services.swhkd = {
    enable = mkEnableOption "Simple Wayland HotKey Daemon";

    package = mkOption {
      description = "The package to use for `swhkd`";
      default = self.packages.${system}.default;
      type = types.package;
    };

    cooldown = mkOption {
      description = "The cooldown to use for `swhkd`";
      default = 250;
      type = types.int;
    };

    settings = mkOption {
      description = "The config to use for `swhkd` syntax and samples could found in [repo](https://github.com/waycrate/swhkd).";
      default = ''
      super + return
        alacritty
      '';
      type = types.lines;
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    environment.etc."swhkd/swhkdrc".text = cfg.settings;

    systemd.user.services.swhkd = {
      description = "Simple Wayland HotKey Daemon";
      bindsTo = [ "default.target" ];
      script = ''
        /run/wrappers/bin/pkexec ${cfg.package}/bin/swhkd \
          --cooldown ${toString cfg.cooldown}
      '';
      serviceConfig.Restart = "always";
      wantedBy = [ "default.target" ];
    };
    security.polkit = {
      enable = true;
      extraConfig = ''
        polkit.addRule(function(action, subject) {
            if (action.id == "com.github.swhkd.pkexec"  &&
                subject.local == true &&
                subject.active == true &&) {
                    return polkit.Result.YES;
                }
        });
      '';
    };
  };
}
