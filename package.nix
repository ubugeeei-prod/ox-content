{
  craneLib,
  lib,
  libiconv,
  pkg-config,
  root ? ./.,
  stdenv,
}:
let
  inherit (lib.importTOML (root + /Cargo.toml)) workspace;
  inherit (workspace.package) version homepage license;

  # Installed binary -> the crate that produces it. `ox_content_napi` and
  # `ox_content_wasm` are deliberately absent: one links against the Node
  # headers napi-build expects and the other only makes sense for the wasm32
  # target, so neither builds as a host binary and neither is what `nix run`
  # should hand you.
  #
  # One mapping rather than two parallel lists, because the build selects
  # crates and `meta` describes binaries, and the pairing between them is what
  # would rot if they were kept apart.
  binaries = {
    "ox-content-i18n" = "ox_content_i18n_cli";
    "ox-content-i18n-lsp" = "ox_content_i18n_lsp";
    "ox-content-link-check" = "ox_content_link_checker";
    "ox-content-lsp" = "ox_content_lsp";
    "ox-content-mdc-check" = "ox_content_mdc_checker";
    "ox-content-profile" = "ox_content_profile_cli";
  };

  mainProgram = "ox-content-lsp";

  # Every binary crate states its own description; none of them are restated
  # here.
  describe = crate: (lib.importTOML (root + "/crates/${crate}/Cargo.toml")).package.description;

  # `cleanCargoSource` keeps only what Cargo itself reads, but a lot of this
  # workspace is embedded at compile time with `include_str!`: tree-sitter
  # highlight queries, the i18n runtime, the SSG's stylesheets and scripts, the
  # HTML entity table, the not-by-ai badges. Two crates reach outside `crates/`
  # as well, for a lint dictionary that ships with the Vite plugin and for a
  # benchmark corpus. Filtered out, they fail the build during macro expansion
  # rather than at link time, which is why the error names a file and not a
  # symbol.
  #
  # `.snap` is deliberately not in the list. Insta snapshots are read only by
  # tests, which `doCheck = false` skips, and there are several hundred of them.
  crateAssetSuffixes = [
    ".css"
    ".html"
    ".js"
    ".json"
    ".md"
    ".scm"
    ".svg"
    ".ts"
    ".txt"
  ];

  externalAssets = [
    "/benchmarks/bundle-size/content/api.md"
    "/npm/vite-plugin-ox-content/src/lint-dictionaries.json"
  ];

  src = lib.cleanSourceWith {
    name = "ox-content-source";
    src = lib.cleanSource root;
    filter =
      path: type:
      craneLib.filterCargoSources path type
      || (lib.hasInfix "/crates/" path && lib.any (suffix: lib.hasSuffix suffix path) crateAssetSuffixes)
      || lib.any (asset: lib.hasSuffix asset path) externalAssets;
  };

  commonArgs = {
    pname = "ox-content";
    inherit version src;
    strictDeps = true;

    # The workspace's tests are driven by the Vite+ task graph (`vp run
    # test:rust`), which knows about the fixtures and the corpora this
    # derivation does not carry.
    doCheck = false;

    cargoExtraArgs = lib.concatMapStringsSep " " (crate: "-p ${crate}") (lib.attrValues binaries);

    nativeBuildInputs = [ pkg-config ];
    buildInputs = lib.optionals stdenv.hostPlatform.isDarwin [ libiconv ];
  };

  # Built once from a manifest-only source tree and reused by every later
  # build, so editing a crate does not rebuild oxc and its dependents.
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    passthru = {
      inherit binaries cargoArtifacts commonArgs;
    };

    meta = {
      # The derivation carries six binaries, so `description` follows the main
      # program and the others are listed rather than summarised into a phrase
      # that no manifest says.
      description = describe binaries.${mainProgram};
      longDescription = ''
        Ox Content's language servers and command-line tools:

        ${lib.concatStringsSep "\n" (
          lib.mapAttrsToList (name: crate: "  ${name} - ${describe crate}") binaries
        )}
      '';
      inherit homepage mainProgram;
      license = lib.getLicenseFromSpdxId license;
      platforms = import (root + /nix/systems.nix);
    };
  }
)
