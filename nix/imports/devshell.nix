{ ... }:

{
  perSystem =
    { pkgs, config, ... }:
    {
      pre-commit.settings.hooks = {
        commitizen.enable = true;
        ruff.enable = true;
        ruff-format.enable = true;
      };

      devShells.default = pkgs.mkShell {
        venvDir = "./.venv";

        packages = [
          # This execute some shell code to initialize a venv in $venvDir before
          # dropping into the shell
          pkgs.python3Packages.venvShellHook
          # UV for Python dependency management
          pkgs.uv
          # Ruff for python code analysis and code formatting
          pkgs.ruff
        ];

        # Run this command, only after creating the virtual environment
        postVenvCreation = ''
          uv sync --dev
          ${config.pre-commit.installationScript}
        '';

        env = {
          PYTHON_KEYRING_BACKEND = "keyring.backends.null.Keyring";
          LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib/";
        };
      };
    };
}
