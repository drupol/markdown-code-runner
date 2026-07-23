{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
    inputs.pedantix.flakeModules.default
  ];

  perSystem =
    { pkgs, ... }:
    {
      treefmt = {
        programs = {
          deadnix.enable = true;
          jsonfmt.enable = true;

          nixfmt = {
            enable = true;
            package = pkgs.nixfmt-rs;
          };

          oxfmt.enable = true;
          pedantix.enable = true;
          statix.enable = true;
          typos.enable = true;
          yamlfmt.enable = true;
        };

        projectRootFile = "flake.nix";

        settings = {
          no-cache = true;
          on-unmatched = "warn";
        };
      };
    };
}
