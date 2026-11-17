//! The manifest, as the core at the pinned commit reads it.

use toml::{Table, Value};

fn manifest() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/plugin.toml")).unwrap_or_default()
}

#[test]
fn the_pinned_core_refuses_nothing_in_the_manifest() {
    let read: Result<lemonfiber_plugin::Manifest, _> = toml::from_str(&manifest());
    let refused: Vec<String> = match read {
        Ok(manifest) => lemonfiber_plugin::refusals(&manifest, &[])
            .into_iter()
            .map(|violation| format!("{}: {}", violation.location, violation.message))
            .collect(),
        Err(why) => vec![why.to_string()],
    };
    assert!(refused.is_empty(), "{refused:#?}");
}

/// The digest of the service named for the plugin, which is the upstream the adapter fronts.
fn upstream_digest(manifest: &Table) -> Option<String> {
    let id = manifest
        .get("plugin")
        .and_then(Value::as_table)
        .and_then(|plugin| plugin.get("id"))
        .and_then(Value::as_str)?;
    manifest
        .get("service")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_table)
        .find(|service| service.get("id").and_then(Value::as_str) == Some(id))
        .and_then(|service| service.get("digest"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

#[test]
fn the_adapter_names_the_image_of_the_configarr_it_fronts() {
    let named: Vec<String> = configarr_adapter::adapter::about()
        .releases
        .into_iter()
        .map(|release| release.digest)
        .collect();
    let upstream = upstream_digest(&manifest().parse().unwrap_or_default());
    assert_eq!(
        upstream.as_deref(),
        Some(configarr_adapter::adapter::DIGEST)
    );
    assert_eq!(named, upstream.into_iter().collect::<Vec<_>>());
}
