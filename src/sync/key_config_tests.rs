use super::key_config::*;
use std::env;
use std::sync::Mutex;
use zeroize::Zeroizing;

const TEST_KEY_BYTES: [u8; KEY_BYTES_LEN] = [7u8; KEY_BYTES_LEN];
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn test_key_hex() -> String {
    format!("hex:{}", hex::encode(TEST_KEY_BYTES))
}

fn test_key_hex_unprefixed() -> String {
    hex::encode(TEST_KEY_BYTES)
}

fn inline_material(value: String) -> EncodedKeyMaterial {
    EncodedKeyMaterial::Inline(Zeroizing::new(value))
}

fn clear_key_env() {
    env::remove_var(KEY_ENV);
    env::remove_var(KEY_FILE_ENV);
    env::remove_var(PREVIOUS_KEY_ENV);
    env::remove_var(PREVIOUS_KEY_FILE_ENV);
    env::remove_var(PREVIOUS_KEY_ID_ENV);
}

#[test]
fn configured_key_reload_yields_identical_key_id_and_metadata() {
    // Given: the same durable test key material loaded twice.
    let material_a = inline_material(test_key_hex());
    let material_b = inline_material(test_key_hex());

    // When: providers are built as if across process restarts.
    let provider_a =
        ConfiguredCloudKeyProvider::from_material(material_a, Some("sha256:old".to_string()))
            .expect("test key config loads");
    let provider_b =
        ConfiguredCloudKeyProvider::from_material(material_b, Some("sha256:old".to_string()))
            .expect("test key config reloads");

    // Then: the key ID and rotation metadata are deterministic.
    assert_eq!(provider_a.active_key().id(), provider_b.active_key().id());
    assert_eq!(
        provider_a.rotation_metadata(),
        provider_b.rotation_metadata()
    );
    assert_eq!(provider_a.rotation_metadata().format_version, 1);
    assert_eq!(provider_a.rotation_metadata().algorithm, "AES-256-GCM");
}

#[test]
fn missing_and_malformed_keys_fail_closed_with_redacted_errors() {
    // Given/When: missing and malformed key sources are parsed.
    let missing = ConfiguredCloudKeyProvider::from_material(inline_material(String::new()), None)
        .expect_err("empty material is malformed");
    let malformed = ConfiguredCloudKeyProvider::from_material(
        inline_material("base64:not-a-32-byte-key".to_string()),
        None,
    )
    .expect_err("short material is malformed");

    // Then: failures are explicit and do not echo input material.
    assert!(matches!(missing, KeyConfigError::MalformedKey));
    assert!(matches!(malformed, KeyConfigError::MalformedKey));
    assert!(!format!("{missing:?}").contains("not-a-32-byte-key"));
    assert!(!malformed.to_string().contains("not-a-32-byte-key"));
}

#[test]
fn debug_output_never_contains_key_bytes() {
    // Given: a configured key with recognizable encoded and raw bytes.
    let provider = ConfiguredCloudKeyProvider::from_material(inline_material(test_key_hex()), None)
        .expect("test key config loads");

    // When: debug output is generated for the provider and active key.
    let provider_debug = format!("{provider:?}");
    let key_debug = format!("{:?}", provider.active_key());

    // Then: the stable key ID is visible but raw/encoded key material is redacted.
    assert!(provider_debug.contains(provider.active_key().id().as_str()));
    assert!(!provider_debug.contains(&hex::encode(TEST_KEY_BYTES)));
    assert!(!key_debug.contains("[7"));
    assert!(key_debug.contains("<redacted>"));
}

#[test]
fn key_file_source_loads_same_config_as_inline_source() {
    // Given: a key file containing the documented encoded key material.
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let key_path = temp_dir.path().join("cloud.key");
    std::fs::write(&key_path, test_key_hex()).expect("test key file is written");

    // When: providers load inline and file-backed sources.
    let inline = ConfiguredCloudKeyProvider::from_material(inline_material(test_key_hex()), None)
        .expect("inline key config loads");
    let file = ConfiguredCloudKeyProvider::from_material(EncodedKeyMaterial::File(key_path), None)
        .expect("file key config loads");

    // Then: both sources produce the same active key ID.
    assert_eq!(inline.active_key().id(), file.active_key().id());
}

#[test]
fn unprefixed_hex_inline_and_file_sources_match_prefixed_hex() {
    // Given: the same documented 64-character hex key in prefixed, inline, and file forms.
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let key_path = temp_dir.path().join("cloud.key");
    std::fs::write(&key_path, test_key_hex_unprefixed()).expect("test key file is written");

    // When: providers parse each supported source.
    let prefixed = ConfiguredCloudKeyProvider::from_material(inline_material(test_key_hex()), None)
        .expect("prefixed hex loads");
    let inline =
        ConfiguredCloudKeyProvider::from_material(inline_material(test_key_hex_unprefixed()), None)
            .expect("unprefixed inline hex loads");
    let file = ConfiguredCloudKeyProvider::from_material(EncodedKeyMaterial::File(key_path), None)
        .expect("unprefixed file hex loads");

    // Then: all forms decode to the same stable key identifier.
    assert_eq!(prefixed.active_key().id(), inline.active_key().id());
    assert_eq!(prefixed.active_key().id(), file.active_key().id());
}

#[test]
fn env_config_missing_key_source_fails_closed() {
    let _guard = ENV_LOCK.lock().expect("key env test lock is not poisoned");
    // Given: no documented key source env vars are set.
    clear_key_env();

    // When: the provider loads from the process environment.
    let error = ConfiguredCloudKeyProvider::from_env().expect_err("missing source must fail");

    // Then: the error names the required config sources and no secret material.
    let message = error.to_string();
    assert!(matches!(error, KeyConfigError::MissingSource));
    assert!(message.contains(KEY_ENV));
    assert!(message.contains(KEY_FILE_ENV));
    clear_key_env();
}

#[test]
fn env_config_rejects_ambiguous_sources_without_echoing_key() {
    let _guard = ENV_LOCK.lock().expect("key env test lock is not poisoned");
    // Given: both inline and file-backed key source env vars are set.
    env::set_var(KEY_ENV, test_key_hex());
    env::set_var(KEY_FILE_ENV, "/tmp/engram-key");

    // When: the provider loads from the process environment.
    let error = ConfiguredCloudKeyProvider::from_env().expect_err("ambiguous sources must fail");

    // Then: the error is explicit and does not echo inline key material.
    let message = error.to_string();
    assert!(matches!(error, KeyConfigError::AmbiguousSources));
    assert!(message.contains("ambiguous"));
    assert!(!message.contains(&hex::encode(TEST_KEY_BYTES)));
    clear_key_env();
}

#[test]
fn env_config_rejects_ambiguous_sources_before_parsing_invalid_material() {
    let _guard = ENV_LOCK.lock().expect("key env test lock is not poisoned");
    // Given: both documented sources are set, and both contain invalid material.
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let key_path = temp_dir.path().join("cloud.key");
    std::fs::write(&key_path, "also-not-a-key").expect("test key file is written");
    clear_key_env();
    env::set_var(KEY_ENV, "not-a-key");
    env::set_var(KEY_FILE_ENV, &key_path);

    // When: the provider loads from the process environment.
    let error = ConfiguredCloudKeyProvider::from_env().expect_err("ambiguous sources must fail");

    // Then: source ambiguity wins and neither invalid secret value is echoed.
    let message = error.to_string();
    assert!(matches!(error, KeyConfigError::AmbiguousSources));
    assert!(!message.contains("not-a-key"));
    assert!(!message.contains("also-not-a-key"));
    clear_key_env();
}

#[test]
fn env_config_loads_key_file_and_previous_key_metadata() {
    let _guard = ENV_LOCK.lock().expect("key env test lock is not poisoned");
    // Given: the documented key file env var and previous key metadata are set.
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let key_path = temp_dir.path().join("cloud.key");
    std::fs::write(&key_path, test_key_hex()).expect("test key file is written");
    clear_key_env();
    env::set_var(KEY_FILE_ENV, &key_path);
    env::set_var(PREVIOUS_KEY_ID_ENV, "sha256:previous");

    // When: the provider loads from the process environment.
    let provider = ConfiguredCloudKeyProvider::from_env().expect("file env config loads");

    // Then: active and rotation metadata are populated deterministically.
    assert_eq!(
        provider.rotation_metadata().active_key_id,
        provider.active_key().id().as_str()
    );
    assert_eq!(
        provider.rotation_metadata().previous_key_id.as_deref(),
        Some("sha256:previous")
    );
    clear_key_env();
}

#[test]
fn env_config_loads_previous_key_for_controlled_rotation() {
    let _guard = ENV_LOCK.lock().expect("key env test lock is not poisoned");
    // Given: active and previous key material are configured for a rotation window.
    clear_key_env();
    env::set_var(KEY_ENV, test_key_hex());
    let previous_material = format!("hex:{}", hex::encode([8u8; KEY_BYTES_LEN]));
    env::set_var(PREVIOUS_KEY_ENV, &previous_material);

    // When: the provider is reconstructed from process configuration.
    let provider = ConfiguredCloudKeyProvider::from_env().expect("rotation config loads");
    let previous_key_id = provider
        .rotation_metadata()
        .previous_key_id
        .as_deref()
        .expect("previous key id is derived");

    // Then: the previous key is readable while the active key remains the write key.
    assert!(provider.key_for_id(previous_key_id).is_some());
    assert_ne!(previous_key_id, provider.active_key().id().as_str());
    assert!(!format!("{provider:?}").contains(&hex::encode([8u8; KEY_BYTES_LEN])));
    clear_key_env();
}

#[test]
fn env_config_loads_unprefixed_hex_from_inline_and_file_sources() {
    let _guard = ENV_LOCK.lock().expect("key env test lock is not poisoned");
    // Given: the documented env and file sources contain unprefixed 64-character hex.
    let temp_dir = tempfile::tempdir().expect("test tempdir is created");
    let key_path = temp_dir.path().join("cloud.key");
    std::fs::write(&key_path, test_key_hex_unprefixed()).expect("test key file is written");

    // When: each source is loaded separately from the process environment.
    clear_key_env();
    env::set_var(KEY_ENV, test_key_hex_unprefixed());
    let inline = ConfiguredCloudKeyProvider::from_env().expect("inline env hex loads");
    clear_key_env();
    env::set_var(KEY_FILE_ENV, &key_path);
    let file = ConfiguredCloudKeyProvider::from_env().expect("file env hex loads");

    // Then: both env forms decode to the same stable key identifier.
    assert_eq!(inline.active_key().id(), file.active_key().id());
    clear_key_env();
}
