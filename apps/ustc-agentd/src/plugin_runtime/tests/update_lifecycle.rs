use super::*;
fn version(runtime: &PluginRuntime, root: &Path, version: &str) -> RuntimePackage {
    let request:PluginImportPreviewDto=serde_json::from_value(json!({"schema":"plugin-import-preview/v1","package_id":"community.version-guide","version":version,
        "display_name":"Version guide","source":"Synthetic reviewed version fixture","skill":format!("---\nname: version-guide\ndescription: Version guide fixture.\n---\nReviewed content version {version}.\n"),"mcp":null})).expect("request");
    let packet = runtime.preview_import(request).expect("review packet");
    fs::create_dir(root).expect("package root");
    for (path, text) in packet.files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("parent")).expect("parent");
        fs::write(path, text).expect("reviewed fixture");
    }
    RuntimePackage::load(root).expect("reviewed version")
}
async fn update(
    runtime: &PluginRuntime,
    owned: &(TenantId, UserId),
    request_id: &str,
    intent: Value,
) -> Result<PluginUpdateViewDto, PluginError> {
    runtime
        .update(
            &owned.0,
            &owned.1,
            serde_json::from_value(
                json!({"schema":"plugin-update/v1","request_id":request_id,"intent":intent}),
            )
            .expect("update intent"),
        )
        .await
}
#[tokio::test]
async fn exact_version_update_rollback_and_restart_invalidate_old_grants_and_preserve_retries() {
    let temp = TempRoot::new();
    let owned = owner("version");
    let helper = skill_runtime(temp.0.join("helper/authority.bin"));
    let old = version(&helper, &temp.0.join("old"), "0.1.0");
    let new = version(&helper, &temp.0.join("new"), "0.2.0");
    let runtime = PluginRuntime::with_packages(temp.state(), vec![old.clone(), new.clone()])
        .expect("runtime");
    assert!(install(&runtime, &owned).await.accepted);
    let installed = snapshot(&runtime, &owned).await;
    let checked = probe(&runtime, &owned, &installed)
        .await
        .expect("old probe");
    let enabled = grant_enable(&runtime, &owned, &installed, &checked).await;
    let frozen = runtime
        .session(&owned.0, &owned.1)
        .await
        .expect("old session");
    assert!(command(&runtime,&owned,"disable-before-update",json!({"action":"disable","installation_id":enabled.id,"expected_revision":enabled.revision})).await.expect("disable").accepted);
    let disabled = snapshot(&runtime, &owned).await;
    let preview=update(&runtime,&owned,"review-version",json!({"action":"preview","installation_id":disabled.id,"expected_revision":disabled.revision,"target_version":"0.2.0"})).await.expect("version preview");
    let intent = json!({"action":"apply","installation_id":disabled.id,"expected_revision":disabled.revision,"target_version":"0.2.0","update_id":preview.update_id,
        "plan_digest":preview.plan_digest,"target_readiness":preview.target_readiness,"rollback_readiness":preview.rollback_readiness});
    let applied = update(&runtime, &owned, "apply-version", intent.clone())
        .await
        .expect("B6 apply");
    assert_eq!(applied.state, "appliedpendingconfirmation");
    assert_eq!(applied.installation_id, disabled.id);
    let target_pin = new.configuration.package_pin();
    assert!(matches!(command(&runtime,&owned,"duplicate-target-install",json!({"action":"install","package_id":target_pin.package_id().as_str(),"version":"0.2.0",
        "catalog_revision":target_pin.catalog_revision().as_str(),"package_digest":target_pin.package_digest().as_str()})).await,Err(PluginError::Conflict)));
    let list = runtime.list(&owned.0, &owned.1).await.expect("list");
    let current = list
        .packages
        .iter()
        .find(|package| package.version == "0.2.0")
        .and_then(|package| package.installation.as_ref())
        .expect("target installation");
    assert_eq!(current.state, "disabled");
    assert!(current.active_capabilities.is_empty());
    for name in frozen.bindings.keys() {
        assert_eq!(
            runtime
                .execute_frozen(&frozen, name, json!({}))
                .await
                .status(),
            ChatToolStatus::Denied
        );
    }
    assert!(
        update(&runtime, &owned, "apply-version", intent.clone())
            .await
            .expect("exact retry")
            .replayed
    );
    let mut changed = intent.clone();
    changed["plan_digest"] = json!(Sha256Digest::from_bytes(b"different").as_str());
    assert!(matches!(
        update(&runtime, &owned, "apply-version", changed).await,
        Err(PluginError::Conflict)
    ));
    {
        let state = runtime.state.lock().await;
        let mut stored: Value =
            serde_json::from_slice(&state.authority.updates.encode().expect("update journal"))
                .expect("journal JSON");
        stored["frames"] = json!([]);
        stored["digest"] = json!(Sha256Digest::from_bytes(b"[]").as_str());
        assert!(
            ustc_campus_agent_core::market::update::application::UpdateJournal::decode(
                &serde_json::to_vec(&stored).expect("modified journal"),
                &state.authority.installations,
                &state.authority.grants
            )
            .is_err(),
            "orphaned coupled events cannot survive missing update frames"
        );
    }
    drop(frozen);
    drop(runtime);
    let runtime = PluginRuntime::with_packages(temp.state(), vec![old.clone(), new.clone()])
        .expect("update restart");
    assert!(
        update(&runtime, &owned, "apply-version", intent)
            .await
            .expect("durable exact retry")
            .replayed
    );
    assert!(
        runtime
            .session(&owned.0, &owned.1)
            .await
            .expect("disabled session")
            .definitions()
            .is_empty()
    );
    let current = runtime
        .list(&owned.0, &owned.1)
        .await
        .expect("list")
        .packages
        .into_iter()
        .find(|package| package.version == "0.2.0")
        .and_then(|package| package.installation)
        .expect("current");
    let review=update(&runtime,&owned,"review-rollback",json!({"action":"review_rollback","installation_id":current.id,"expected_revision":current.revision,"update_id":applied.update_id})).await.expect("rollback probe");
    let rollback_intent = json!({"action":"rollback","installation_id":current.id,"expected_revision":current.revision,"update_id":applied.update_id,"rollback_readiness":review.rollback_readiness});
    let rolled_back = update(
        &runtime,
        &owned,
        "rollback-version",
        rollback_intent.clone(),
    )
    .await
    .expect("B6 rollback");
    assert_eq!(rolled_back.state, "rolledback");
    assert_eq!(snapshot(&runtime, &owned).await.state, "disabled");
    assert!(
        snapshot(&runtime, &owned)
            .await
            .active_capabilities
            .is_empty()
    );
    drop(runtime);
    let runtime =
        PluginRuntime::with_packages(temp.state(), vec![old, new]).expect("rollback restart");
    assert!(
        update(&runtime, &owned, "rollback-version", rollback_intent)
            .await
            .expect("rollback retry")
            .replayed
    );
    assert_journal(&runtime, 0).await;
    let bytes = fs::read(temp.state()).expect("state");
    assert!(bytes.starts_with(b"uca-plugin-authority/v2\0"));
}
