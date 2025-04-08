{
  lib,
  python3Packages,
}:

python3Packages.buildPythonPackage {
  pname = "markdown-code-runner";
  version = "0.1.0";
  pyproject = true;

  src = ../../..;

  nativeBuildInputs = with python3Packages; [
    hatchling
  ];

  dependencies = with python3Packages; [ markdown-it-py pydantic-settings ];

  pythonImportsCheck = [ "markdown_code_runner" ];

  nativeCheckInputs = with python3Packages; [
    pytestCheckHook
  ];

  meta = {
    description = "A configurable Markdown code runner that executes and optionally replaces code blocks using external commands";
    longDescription = ''
      markdown-code-runner is a command-line tool that scans Markdown files for fenced code blocks,
      executes them using per-language configuration, and optionally replaces the block content
      with the command output.

      It is useful for documentation that stays in sync with linters, formatters, or scripts.
      The tool supports placeholder substitution, configurable replace/check modes, and CI-friendly validation.
    '';
    homepage = "https://github.com/drupol/markdown-code-runner";
    license = lib.licenses.eupl12;
    mainProgram = "mdcr";
    maintainers = with lib.maintainers; [ drupol ];
    platforms = lib.platforms.all;
  };

}
