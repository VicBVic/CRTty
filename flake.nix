{
  description = "Post-processing shader framework for kitty terminal via LD_PRELOAD";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "crtty";
          version = "0.1.3";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.makeWrapper ];

          installPhase = ''
            runHook preInstall
            mkdir -p $out/bin $out/lib
            cp target/release/crtty   $out/bin/
            cp target/release/libcrtty_crt.so $out/lib/
            cp crtty.conf.example $out/share/doc/crtty/crtty.conf.example 2>/dev/null || true
            runHook postInstall
          '';

          meta = with pkgs.lib; {
            description = "Post-processing shader framework for kitty terminal via LD_PRELOAD";
            homepage = "https://github.com/kosa12/CRTty";
            license = licenses.mit;
            platforms = platforms.linux;
            mainProgram = "crtty";
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
        };
      });
}
