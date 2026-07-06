{
  description = "Terminal-native, markdown-driven AI security operating environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rustfmt" "clippy" ];
        };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "raskolnikov";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ rustToolchain pkg-config ];
          buildInputs = [ pkgs.openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          doCheck = true;
          meta = with pkgs.lib; {
            description = "Terminal-native, markdown-driven AI security operating environment";
            homepage = "https://github.com/dysshanks/Raskolnikov";
            license = licenses.asl20;
            mainProgram = "raskolnikov";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.openssl
            pkgs.nmap
            pkgs.gobuster
            pkgs.nikto
            pkgs.sqlmap
          ];
          shellHook = ''
            echo "Raskolnikov dev shell"
          '';
        };
      });
}
