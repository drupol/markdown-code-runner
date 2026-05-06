{ inputs, withSystem, ... }:
{
  imports = [
    inputs.pkgs-by-name-for-flake-parts.flakeModule
  ];

  perSystem =
    { config, ... }:
    {
      pkgsDirectory = ../pkgs;
      packages.default = config.packages.markdown-code-runner;
    };

  flake = {
    overlays.default =
      _final: prev:
      withSystem prev.stdenv.hostPlatform.system (
        { config, ... }:
        {
          inherit (config.packages) markdown-code-runner;
        }
      );
  };
}
