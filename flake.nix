{
  description = "Export FIT files from Garmin Connect";

  inputs.nixpkgs.url = "nixpkgs/nixos-24.11";

  outputs = { self, nixpkgs }:

    let

      supportedSystems = [ "aarch64-darwin" "x86_64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);

    in

    {
      overlay = final: prev: {
        garmin_export = final.rustPlatform.buildRustPackage rec {
          pname = "garmin_export";
          version = "0.1.0";

          src = builtins.path {
            path = ./.;
            name = pname;
            filter = final.lib.cleanSourceFilter;
          };

          cargoHash = "sha256-BZrthdQkEygcI1XoEgK/EywWQWl/cY8Y3teDTwNA2sM=";

          nativeBuildInputs = [
            final.openssl
            final.pkg-config
          ];

          buildInputs = [
            final.darwin.apple_sdk.frameworks.Security
          ];
        };
      };

      devShell = forAllSystems (system:
        with import nixpkgs { inherit system; };

        mkShell {
          buildInputs = [
            cargo
            clippy
            openssl
            pkg-config
            rustc
            rustfmt
          ];
        }
      );

      defaultPackage = forAllSystems (system:
        (import nixpkgs {
          inherit system;
          overlays = [ self.overlay ];
        }).garmin_export
      );

      defaultApp = forAllSystems (system:
        {
          type = "app";
          program = "${self.defaultPackage.${system}}/bin/garmin_export";
        }
      );
    };
}
