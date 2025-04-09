{
  lib,
  stdenvNoCC,
  rustfmt,
  cargo,
}:

stdenvNoCC.mkDerivation {
  pname = "cargo-fmt";
  version = "1.0.0";

  src = ../../..;

  buildInputs = [ rustfmt ];

  buildPhase = ''
    ${lib.getExe cargo} fmt -- --check | tee $out
  '';
}
