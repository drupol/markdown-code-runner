{ inputs, withSystem, ... }:
{
  imports = [
    inputs.pkgs-by-name-for-flake-parts.flakeModule
  ];

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

  perSystem =
    { config, ... }:
    {
      packages.default = config.packages.markdown-code-runner;
      pkgsDirectory = ../pkgs;
    };
}
