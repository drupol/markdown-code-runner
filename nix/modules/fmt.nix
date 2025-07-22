{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem = {
    treefmt = {
      projectRootFile = "flake.nix";
      programs = {
        jsonfmt.enable = true;
        nixfmt.enable = true;
        prettier.enable = true;
        rustfmt.enable = true;
        statix.enable = true;
        typos.enable = true;
        yamlfmt.enable = true;
      };
      settings = {
        on-unmatched = "fatal";
        global.excludes = [
          "*.envrc"
          ".editorconfig"
          ".prettierrc"
          "*.crt"
          "*.directory"
          "*.face"
          "*.fish"
          "*.png"
          "*.toml"
          "*.svg"
          "*.xml"
          "*/.gitignore"
          "LICENSE"
        ];
      };
    };
  };
}
