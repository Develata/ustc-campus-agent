//! Argument correction never changes the owner authority or retries an effect.
use super::*;

#[tokio::test]
async fn wrong_skill_path_is_recoverable_without_new_grant() {
    let temp = TempRoot::new();
    let owned = owner("argument-recovery");
    let runtime = skill_runtime(temp.state());
    assert!(install(&runtime, &owned).await.accepted);
    let installed = configure(&runtime, &owned, json!({})).await;
    let checked = probe(&runtime, &owned, &installed).await.expect("probe");
    let enabled = grant_enable(&runtime, &owned, &installed, &checked).await;
    let name = projected_name(&runtime, &owned).await;
    let session = runtime.session(&owned.0, &owned.1).await.expect("session");
    let rejected = runtime
        .execute_frozen(&session, &name, json!({"resource":"SKILL.md"}))
        .await;
    assert_eq!(rejected.status(), ChatToolStatus::Failed);
    let payload: Value =
        serde_json::from_str(&rejected.serialize_for_provider().expect("safe result"))
            .expect("JSON");
    assert_eq!(payload["data"]["code"], "plugin_invalid_arguments");
    assert_eq!(snapshot(&runtime, &owned).await.revision, enabled.revision);
    let corrected = runtime.execute_frozen(&session, &name, json!({})).await;
    assert_eq!(corrected.status(), ChatToolStatus::Succeeded);
    let payload: Value =
        serde_json::from_str(&corrected.serialize_for_provider().expect("safe result"))
            .expect("JSON");
    assert_eq!(payload["data"]["resource"], "skills/campus-guide/SKILL.md");
    assert_eq!(payload["data"]["instruction_authority"], "none");
    assert_journal(&runtime, 2).await;
    command(&runtime, &owned, "disable-after-read", json!({"action":"disable", "installation_id":enabled.id, "expected_revision":enabled.revision})).await.expect("disable");
    assert_eq!(
        runtime
            .execute_frozen(&session, &name, json!({}))
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_journal(&runtime, 2).await;
}
