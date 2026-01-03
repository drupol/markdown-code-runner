{
  inputs = {
    nixpkgs.url = "github:/nixos/nixpkgs/nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    flake-compat.url = "github:NixOS/flake-compat";
    flake-compat.flake = false;

    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";

    pkgs-by-name-for-flake-parts.url = "github:drupol/pkgs-by-name-for-flake-parts";

    import-tree.url = "github:vic/import-tree";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    make-shell.url = "github:nicknovitski/make-shell";
  };

  outputs =
    inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } (inputs.import-tree ./nix/modules);
}
