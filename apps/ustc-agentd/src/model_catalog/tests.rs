use super::*;
use serde_json::json;

#[test]
fn closed_catalog_limits_and_duplicate_fields_reject_without_fallback() {
    for bytes in [
        r#"{"schema":"uca-agent-models/v1","models":[],"extra":true}"#,
        r#"{"schema":"uca-agent-models/v1","schema":"uca-agent-models/v1","models":[]}"#,
        r#"{"schema":"uca-agent-models/v1","models":[{"id":"default","label":"x","mode":"mock"}]}"#,
        r#"{"schema":"uca-agent-models/v1","models":[{"id":"x","id":"y","label":"x","mode":"mock"}]}"#,
        r#"{"schema":"uca-agent-models/v1","models":[{"id":"x","label":"x","mode":"mock","mode":"mock"}]}"#,
        r#"{"schema":"uca-agent-models/v1","models":[{"id":"x","label":"x","mode":"mock","base_url":null}]}"#,
        r#"{"schema":"uca-agent-models/v1","models":[{"id":"x","label":"x","mode":"local-chat"}]}"#,
        r#"{"schema":"uca-agent-models/v2","models":[]}"#,
    ] {
        assert!(
            ModelCatalog::from_bytes(ChatProvider::deterministic_mock(), bytes.as_bytes()).is_err(),
            "{bytes}"
        );
    }
    for (id, label) in [
        ("".to_owned(), "label".to_owned()),
        ("x".repeat(65), "label".to_owned()),
        ("space id".to_owned(), "label".to_owned()),
        ("a".into(), " ".into()),
        ("a".into(), "line\nbreak".into()),
        ("a".into(), "长".repeat(43)),
    ] {
        let bytes=json!({"schema":"uca-agent-models/v1","models":[{"id":id,"label":label,"mode":"mock"}]}).to_string();
        assert!(
            ModelCatalog::from_bytes(ChatProvider::deterministic_mock(), bytes.as_bytes()).is_err()
        );
    }
    let too_many = (0..16)
        .map(|i| json!({"id":format!("id{i}"),"label":"mock","mode":"mock"}))
        .collect::<Vec<_>>();
    assert!(
        ModelCatalog::from_bytes(
            ChatProvider::deterministic_mock(),
            json!({"schema":"uca-agent-models/v1","models":too_many})
                .to_string()
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        ModelCatalog::from_bytes(ChatProvider::deterministic_mock(), &vec![b' '; 65537]).is_err()
    );
    let dup = json!({"schema":"uca-agent-models/v1","models":[{"id":"a","label":"a","mode":"mock"},{"id":"a","label":"b","mode":"mock"}]});
    assert!(
        ModelCatalog::from_bytes(
            ChatProvider::deterministic_mock(),
            dup.to_string().as_bytes()
        )
        .is_err()
    );
}

#[test]
fn catalog_view_is_allowlisted_and_model_selection_is_presence_sensitive() {
    let models=ModelCatalog::from_bytes(ChatProvider::deterministic_mock(),br#"{"schema":"uca-agent-models/v1","models":[{"id":"synthetic-a","label":"Synthetic A","mode":"mock"}]}"#).expect("catalog");
    let view = serde_json::to_value(models.view()).expect("public view");
    assert_eq!(view["default_id"], "default");
    assert_eq!(view.as_object().expect("object").len(), 3);
    for entry in view["models"].as_array().expect("entries") {
        assert_eq!(entry.as_object().expect("entry").len(), 5);
        assert_eq!(entry["provider"].as_object().expect("identity").len(), 2);
        assert_eq!(entry["tool_calling"], true);
        assert!(entry["context_limit_tokens"].is_null());
    }
    assert!(models.resolve("unknown").is_err());
    assert!(serde_json::from_str::<ModelSelectionFieldDto>("null").is_err());
    assert!(serde_json::from_str::<ModelSelectionFieldDto>("12").is_err());
    assert_eq!(
        ModelSelectionFieldDto::Absent.selected(false),
        Some("default")
    );
    assert!(ModelSelectionFieldDto::Absent.selected(true).is_none());
    assert!(
        ModelSelectionFieldDto::Value("default".into())
            .selected(false)
            .is_none()
    );
}

#[cfg(unix)]
#[test]
fn model_file_and_credentials_are_regular_bounded_private_and_absolute() {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };
    let root = std::env::temp_dir().join(format!(
        "uca-model-file-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    fs::create_dir(&root).expect("temp");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("private temp");
    let key = root.join("synthetic.key");
    fs::write(&key, "synthetic-model-secret").expect("key");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("private key");
    let file = root.join("models.json");
    let config = json!({"schema":"uca-agent-models/v1","models":[{"id":"local","label":"Local","mode":"local-chat","base_url":"http://127.0.0.1:9/v1","model":"synthetic-model","api_key_file":key,"timeout_ms":1000,"context_limit_tokens":2048}]});
    fs::write(&file, config.to_string()).expect("catalog");
    let models = ModelCatalog::from_file(ChatProvider::deterministic_mock(), &file)
        .expect("no discovery network needed");
    let view = serde_json::to_string(&models.view()).expect("view");
    assert!(
        !view.contains("127.0.0.1")
            && !view.contains("synthetic-model-secret")
            && !view.contains("synthetic.key")
    );
    assert!(
        !models
            .resolve("local")
            .expect("selected")
            .tool_calling_enabled()
    );
    for (field, value) in [
        ("api_key_file", json!("relative.key")),
        ("api_key_file", json!(root.join("missing"))),
        ("timeout_ms", json!(999)),
        ("context_limit_tokens", json!(1)),
        ("base_url", json!("https://remote.example/v1")),
    ] {
        let mut invalid = config.clone();
        invalid["models"][0][field] = value;
        assert!(
            ModelCatalog::from_bytes(
                ChatProvider::deterministic_mock(),
                invalid.to_string().as_bytes()
            )
            .is_err()
        );
    }
    fs::set_permissions(&key, fs::Permissions::from_mode(0o644)).expect("unsafe fixture mode");
    assert!(ModelCatalog::from_file(ChatProvider::deterministic_mock(), &file).is_err());
    let link = root.join("link.json");
    symlink(&file, &link).expect("link");
    assert!(ModelCatalog::from_file(ChatProvider::deterministic_mock(), &link).is_err());
    assert!(ModelCatalog::from_file(ChatProvider::deterministic_mock(), &root).is_err());
    fs::write(&file, vec![b' '; 65537]).expect("oversize");
    assert!(ModelCatalog::from_file(ChatProvider::deterministic_mock(), &file).is_err());
    fs::remove_dir_all(root).expect("remove owned fixture");
}
