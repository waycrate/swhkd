{
  description = "Swhkd devel";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable"; };

  outputs = { self, nixpkgs, ... }:
    let
      pkgsFor = system:
        import nixpkgs {
          inherit system;
          overlays = [ ];
        };

      targetSystems = [ "aarch64-linux" "x86_64-linux" ];
    in {
      devShells = nixpkgs.lib.genAttrs targetSystems (system:
        let pkgs = pkgsFor system;
        in {
          default = pkgs.mkShell {
            name = "Swhkd-devel";
            nativeBuildInputs = with pkgs; [
              # Compilers
              cargo
              rustc
              scdoc

              # Tools
			  zip
              rustfmt
              clippy
              gdb
              gnumake
              rust-analyzer
              strace
              valgrind
            ];
          };
        });
    };
}
