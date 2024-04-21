{
  description = "Swhkd devel";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, ... }:
    let
      pkgsFor = system:
        import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };

      targetSystems = [ "aarch64-linux" "x86_64-linux" ];
    in
    {
      devShells = nixpkgs.lib.genAttrs targetSystems (system:
        let pkgs = pkgsFor system;
        in {
          default = pkgs.mkShell {
            name = "Swhkd-devel";
            nativeBuildInputs = with pkgs; [
              (pkgs.fenix.complete.withComponents [
                "cargo"
                "clippy"
                "rust-src"
                "rustc"
                "rustfmt"
              ])
              # libs
              udev

              # Tools
              pkg-config
              clippy
              gdb
              gnumake
              rust-analyzer-nightly
              rustfmt
              strace
              valgrind
              zip
            ];
          };
        });
      packages = nixpkgs.lib.genAttrs targetSystems (system:
        let
          toolchain = fenix.packages.${system}.minimal.toolchain;
          pkgs = pkgsFor system;
        in
        {
          default = (pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          }).buildRustPackage {
            pname = "swhkd";
            version = "1.2.1";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ udev ];
          };
        });
      overlays = nixpkgs.lib.genAttrs targetSystems (system: {
        default = final: prev: {
          swhkd = self.packages.${system}.default;
        };
      });
    };
}
