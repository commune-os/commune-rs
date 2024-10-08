{
  craneLib,
  inputs,
}: let
  cargoManifest = inputs.pkgs.lib.importTOML ./Cargo.toml;

  buildPackageEnv = {
    COMMUNE_VERSION = inputs.self.shortRev or inputs.self.dirtyShortRev;
  };

  commonArgs = {
    name = cargoManifest.package.name;
    version = cargoManifest.package.version;

    src = inputs.nix-filter.lib {
      include = [
        "Cargo.lock"
        "Cargo.toml"
        "src"
      ];
      root = inputs.self;
    };
  };
in
  craneLib.buildPackage (commonArgs
    // {
      doCheck = false;

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      cargoExtraArgs =
        "--locked --no-default-features "
        + inputs.pkgs.lib.optionalString
        (cargoManifest.features.default != [])
        "--features "
        + (builtins.concatStringsSep "," cargoManifest.features.default);

      env = buildPackageEnv;

      passthru = {
        env = buildPackageEnv;
      };

      meta.mainProgram = commonArgs.name;
    })