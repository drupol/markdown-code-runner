{ ... }:

{
  perSystem =
    { pkgs, ... }:
    {
      pre-commit.settings.hooks = {
        commitizen.enable = true;
      };

      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
            cargo
            clippy
            rust-analyzer
            rustc
            rustfmt
        ];

        shellHook = ''
          export PATH=$PWD/target/debug:$PATH
          export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        '';
      };
    };
}
