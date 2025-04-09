{
  lib,
  cargo,
  clippy,
  markdown-code-runner,
}:

markdown-code-runner.overrideAttrs (oldAttrs: {
  nativeCheckInputs = (oldAttrs.nativeCheckInputs or [ ]) ++ [
    cargo
    clippy
  ];

  checkPhase = ''
    RUSTFLAGS="-Dwarnings" ${lib.getExe cargo} clippy
  '';
})
