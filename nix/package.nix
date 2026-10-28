{
  lib,
  rustPlatform,
  cmake,
  pkg-config,
  zstd,
}:
let
  cargoToml = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  inherit (cargoToml.package) version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../build.rs
      ../src
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    cmake
    pkg-config
  ];
  buildInputs = [ zstd ];

  meta = {
    inherit (cargoToml.package) description;
    homepage = cargoToml.package.repository;
    license = lib.licenses.asl20;
    mainProgram = "lexeme";
  };
}
