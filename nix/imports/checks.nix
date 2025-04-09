{ ... }:

{
  perSystem =
    { config, ... }:
    {
      checks = {
        inherit (config.packages) cargo-clippy cargo-fmt;
      };
    };
}
