{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem = {
    treefmt = {
      projectRootFile = "flake.nix";
      programs = {
        deadnix.enable = true;
        jsonfmt.enable = true;
        nixfmt.enable = true;
        prettier.enable = true;
        statix.enable = true;
        typos.enable = true;
        yamlfmt.enable = true;
      };
      settings = {
        no-cache = true;
        on-unmatched = "warn";
      };
    };
  };
}
