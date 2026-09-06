//! Transport state owns quarantine; application results explain its recovery.
use super::*;

#[tokio::test]
async fn mcp_business_drift_reports_review_and_persists_failed_receipt_without_retry() {
    let temp = TempRoot::new();
    let owned = owner("business-drift");
    let package = write_mcp_package(&temp.0.join("package"));
    let peer = McpPeer::start(temp.state());
    let runtime =
        PluginRuntime::with_packages(temp.state(), vec![package.clone()]).expect("runtime");
    install(&runtime, &owned).await;
    let installed = configure(&runtime, &owned, json!({"endpoint":peer.endpoint})).await;
    let discovered = probe(&runtime, &owned, &installed).await.expect("probe");
    grant_enable(&runtime, &owned, &installed, &discovered).await;
    let session = runtime.session(&owned.0, &owned.1).await.expect("session");
    let name = projected_name(&runtime, &owned).await;
    peer.control.business_fault.store(1, Ordering::SeqCst);
    let result = runtime
        .execute_frozen(&session, &name, json!({"query":"synthetic"}))
        .await;
    let payload: Value =
        serde_json::from_str(&result.serialize_for_provider().expect("safe result")).expect("JSON");
    assert_eq!(payload["data"]["code"], "plugin_review_required");
    assert_eq!(result.status(), ChatToolStatus::Denied);
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 1);
    assert_journal(&runtime, 1).await;
    {
        let state = runtime.state.lock().await;
        let journal = serde_json::to_value(&state.authority.runs[0]).expect("journal");
        assert_eq!(
            journal["events"]
                .as_array()
                .expect("events")
                .last()
                .expect("terminal")["kind"]["type"],
            "failed"
        );
    }
    assert_eq!(
        runtime
            .execute_frozen(&session, &name, json!({"query":"blocked"}))
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 1);
    assert_journal(&runtime, 1).await;
    drop(runtime);
    let reopened = PluginRuntime::with_packages(temp.state(), vec![package]).expect("restart");
    assert_journal(&reopened, 1).await;
    assert_eq!(
        peer.control.calls.load(Ordering::SeqCst),
        1,
        "journal recovery performs no business retry"
    );
}

#[tokio::test]
async fn mcp_ordinary_business_failure_keeps_transient_result_and_current_review() {
    for fault in [2, 3] {
        let temp = TempRoot::new();
        let owned = owner("business-failure");
        let package = write_mcp_package(&temp.0.join("package"));
        let peer = McpPeer::start(temp.state());
        let runtime = PluginRuntime::with_packages(temp.state(), vec![package]).expect("runtime");
        install(&runtime, &owned).await;
        let installed = configure(&runtime, &owned, json!({"endpoint":peer.endpoint})).await;
        let discovered = probe(&runtime, &owned, &installed).await.expect("probe");
        grant_enable(&runtime, &owned, &installed, &discovered).await;
        let session = runtime.session(&owned.0, &owned.1).await.expect("session");
        let name = projected_name(&runtime, &owned).await;
        peer.control.business_fault.store(fault, Ordering::SeqCst);
        let result = runtime
            .execute_frozen(&session, &name, json!({"query":"synthetic"}))
            .await;
        let payload: Value =
            serde_json::from_str(&result.serialize_for_provider().expect("result")).expect("JSON");
        assert_eq!(payload["data"]["code"], "plugin_execution_unavailable");
        assert_eq!(result.status(), ChatToolStatus::Failed);
        assert_eq!(peer.control.calls.load(Ordering::SeqCst), 1);
        assert_journal(&runtime, 1).await;
        assert_eq!(
            runtime
                .execute_frozen(&session, &name, json!({"query":"explicit new attempt"}))
                .await
                .status(),
            ChatToolStatus::Succeeded
        );
        assert_eq!(peer.control.calls.load(Ordering::SeqCst), 2);
        assert_journal(&runtime, 2).await;
    }
}
