use std::path::PathBuf;

use super::*;

#[tokio::test]
async fn spawn_errors_when_the_tsgo_binary_does_not_exist() {
    let config = ResolverConfig::with_tsgo_path("/this/path/should/not/exist/tsgo");
    let err = Resolver::spawn(config).await.expect_err("expected TsgoMissing");
    assert!(matches!(err, Error::TsgoMissing(_)), "got {err:?}");
}

#[tokio::test]
async fn resolve_component_props_rejects_relative_paths() {
    // Pretend tsgo is present so we get past spawn() into the
    // resolver path. The current scaffold returns
    // Error::InvalidComponentPath before doing any I/O.
    let tmp = std::env::temp_dir().join("ox-content-cr-fake-tsgo");
    std::fs::write(&tmp, "").unwrap();
    let resolver = Resolver::spawn(ResolverConfig::with_tsgo_path(&tmp)).await.expect("spawn");

    let err = resolver
        .resolve_component_props(&PathBuf::from("relative.tsx"))
        .await
        .expect_err("expected InvalidComponentPath");
    assert!(matches!(err, Error::InvalidComponentPath { .. }), "got {err:?}");
}

#[tokio::test]
async fn resolve_component_props_returns_not_implemented_until_followup_lands() {
    let tmp = std::env::temp_dir().join("ox-content-cr-fake-tsgo-2");
    std::fs::write(&tmp, "").unwrap();
    let resolver = Resolver::spawn(ResolverConfig::with_tsgo_path(&tmp)).await.expect("spawn");

    let target = std::env::temp_dir().join("Alert.tsx");
    std::fs::write(&target, "").unwrap();
    let err = resolver.resolve_component_props(&target).await.expect_err("scaffold");
    assert!(matches!(err, Error::NotImplemented), "got {err:?}");
}

#[test]
fn resolver_config_carries_tsgo_path_and_optional_tsconfig() {
    let config =
        ResolverConfig::with_tsgo_path("/usr/local/bin/tsgo").with_tsconfig("/repo/tsconfig.json");
    assert_eq!(config.tsgo_path, PathBuf::from("/usr/local/bin/tsgo"));
    assert_eq!(config.tsconfig_path.as_deref(), Some(std::path::Path::new("/repo/tsconfig.json")));
}

#[test]
fn resolved_prop_round_trips_through_serde() {
    let prop = ResolvedProp {
        name: "tone".into(),
        type_text: "'info' | 'warn' | 'error'".into(),
        optional: false,
        docs: Some("Severity of the alert".into()),
        location: Some(Location {
            file: PathBuf::from("/repo/src/Alert.tsx"),
            line: 12,
            column: 3,
        }),
    };
    let json = serde_json::to_string(&prop).expect("serialize");
    assert!(json.contains("\"tone\""));
    assert!(json.contains("'info' | 'warn' | 'error'"));
    assert!(json.contains("\"line\":12"));
}
