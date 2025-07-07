{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgs = forAllSystems (system: nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (system: {
        default = pkgs.${system}.mkShell {
          buildInputs =
            with pkgs.${system};
            let
              pkgs_linux = lib.optionals stdenv.isLinux [
                pkg-config
                openssl
              ];
            in
            [
              cargo
              rustc
              rust-analyzer
              rustfmt
              lldb
              bacon
              cargo-nextest
            ]
            ++ pkgs_linux;
        };
      });
    };
}
