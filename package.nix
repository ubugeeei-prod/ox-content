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

  # The crates that produce binaries. `ox_content_napi` and `ox_content_wasm`
  # are deliberately absent: one links against the Node headers napi-build
  # expects and the other only makes sense for the wasm32 target, so neither
  # builds as a host binary and neither is what `nix run` should hand you.
  binCrates = [
    "ox_content_i18n_cli"
    "ox_content_i18n_lsp"
    "ox_content_link_checker"
    "ox_content_lsp"
    "ox_content_mdc_checker"
    "ox_content_profile_cli"
  ];

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

    cargoExtraArgs = lib.concatMapStringsSep " " (crate: "-p ${crate}") binCrates;

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
      inherit binCrates cargoArtifacts commonArgs;
      binaries = [
        "ox-content-i18n"
        "ox-content-i18n-lsp"
        "ox-content-link-check"
        "ox-content-lsp"
        "ox-content-mdc-check"
        "ox-content-profile"
      ];
    };

    meta = {
      description = "Ox Content language servers and command-line tools";
      inherit homepage;
      license = lib.getLicenseFromSpdxId license;
      mainProgram = "ox-content-lsp";
    };
  }
)
