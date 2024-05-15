{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.cargo;
          rustc = pkgs.rustc;
        };

        lintingRustFlags = "-D unused-crate-dependencies";
      in rec {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            # Rust toolchain
            cargo
            rustc

            # Code analysis tools
            clippy
            rust-analyzer

            # Code formatting tools
            treefmt
            alejandra
            rustfmt
            mdl

            # Rust dependency linting
            cargo-deny

            # Container image management tool
            skopeo
          ];

          RUSTFLAGS = lintingRustFlags;
        };

        packages = rec {
          default = rustPlatform.buildRustPackage {
            pname = "emfcamp-dapnet-schedule-announcer";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;

              outputHashes = {
                "emfcamp-schedule-api-0.0.1" = "sha256-4J+7gwxQNydb9COOL58H+qO8iOTSsX8mW7lTdolH9d8=";
              };
            };

            # Nothing to test
            doCheck = false;
          };

          container-image = pkgs.dockerTools.buildImage {
            name = "emfcamp-dapnet-schedule-announcer";
            tag = "latest";
            created = "now";

            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              paths = [pkgs.bashInteractive pkgs.coreutils];
              pathsToLink = ["/bin"];
            };

            config = {
              Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${default}/bin/emfcamp-dapnet-schedule-announcer"];
              ExposedPorts = {
                "9090/tcp" = {};
              };
              Env = [
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
                "OBSERVABILITY_ADDRESS=0.0.0.0:9090"
              ];
            };
          };
        };
      }
    );
}
