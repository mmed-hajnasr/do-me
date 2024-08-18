{
  description = "just-ball dev enveirement";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {

        packages = with pkgs;[
          cargo
          rustc

          rust-analyzer
          clippy
          rustfmt

          pkg-config
          sqlite
          lldb
        ];

        env = {
          RUST_BACKTRACE = 1;
        };

      };
    };
}

