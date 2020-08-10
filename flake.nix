{
  description = "A very basic flake";

  outputs = { self, nixpkgs }:
    let

      pkgs = import nixpkgs {
        system = "x86_64-linux";
      };

    in

    {
      devShell.x86_64-linux = pkgs.mkShell {
        buildInputs = [
          pkgs.cargo
          pkgs.clippy
          pkgs.openssl.dev
          pkgs.pkg-config
          pkgs.rustc
          pkgs.rustfmt
        ];
      };
    };
}
