//! Stateful peer with a one-session quota exercises actual DELETE integration.
use super::*;

#[tokio::test]
async fn mcp_session_retirement_reprobe_preserves_single_session_quota() {
    let temp = TempRoot::new();
    let owned = owner("quota");
    let package = write_mcp_package(&temp.0.join("package"));
    let peer = McpPeer::start(temp.state());
    peer.control.session_limit.store(1, Ordering::SeqCst);
    let runtime = PluginRuntime::with_packages(temp.state(), vec![package]).expect("runtime");
    install(&runtime, &owned).await;
    let installed = configure(&runtime, &owned, json!({"endpoint":peer.endpoint})).await;
    for _ in 0..3 {
        probe(&runtime, &owned, &installed)
            .await
            .expect("reprobe must release previous quota slot");
    }
    let requests = peer.control.observed.lock().expect("requests");
    assert_eq!(requests.iter().filter(|m| *m == "DELETE").count(), 2);
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn mcp_session_retirement_after_management_preserves_durable_receipts() {
    for action in ["configure", "disable", "revoke"] {
        for fail_delete in [false, true] {
            let temp = TempRoot::new();
            let owned = owner("manage-close");
            let package = write_mcp_package(&temp.0.join("package"));
            let peer = McpPeer::start(temp.state());
            let runtime =
                PluginRuntime::with_packages(temp.state(), vec![package.clone()]).expect("runtime");
            install(&runtime, &owned).await;
            let installed = configure(&runtime, &owned, json!({"endpoint":peer.endpoint})).await;
            let discovered = probe(&runtime, &owned, &installed)
                .await
                .expect("discovery");
            let enabled = if action == "configure" {
                installed
            } else {
                grant_enable(&runtime, &owned, &installed, &discovered).await
            };
            peer.control
                .fail_delete
                .store(fail_delete, Ordering::SeqCst);
            let mut intent = json!({"action":action,"installation_id":enabled.id,"expected_revision":enabled.revision});
            if action == "configure" {
                intent["values"] = json!({"endpoint":format!("{}?updated=1", peer.endpoint)});
            }
            let receipt = command(&runtime, &owned, "retire", intent.clone())
                .await
                .expect("cleanup cannot change management outcome");
            assert!(receipt.accepted, "{action} management accepted");
            assert!(
                runtime
                    .definitions(&owned.0, &owned.1)
                    .await
                    .expect("definitions")
                    .is_empty()
            );
            assert_eq!(
                peer.control
                    .observed
                    .lock()
                    .expect("observed")
                    .iter()
                    .filter(|m| *m == "DELETE")
                    .count(),
                1
            );
            drop(runtime);
            let reopened =
                PluginRuntime::with_packages(temp.state(), vec![package]).expect("reopen");
            let replayed = command(&reopened, &owned, "retire", intent)
                .await
                .expect("exact replay");
            assert!(replayed.accepted && replayed.replayed);
            assert_eq!(replayed.revision, receipt.revision);
            assert_eq!(
                peer.control
                    .observed
                    .lock()
                    .expect("observed")
                    .iter()
                    .filter(|m| *m == "DELETE")
                    .count(),
                1,
                "retry does not repeat cleanup or business effects"
            );
            assert_eq!(peer.control.calls.load(Ordering::SeqCst), 0);
        }
    }
}

#[tokio::test]
async fn mcp_session_retirement_rejected_application_inventory_releases_quota() {
    let temp = TempRoot::new();
    let owned = owner("rejected-discovery");
    let package = write_mcp_package(&temp.0.join("package"));
    let peer = McpPeer::start(temp.state());
    peer.control.session_limit.store(1, Ordering::SeqCst);
    peer.control.unbound_tool.store(true, Ordering::SeqCst);
    let runtime = PluginRuntime::with_packages(temp.state(), vec![package]).expect("runtime");
    install(&runtime, &owned).await;
    let installed = configure(&runtime, &owned, json!({"endpoint":peer.endpoint})).await;
    assert_eq!(
        probe(&runtime, &owned, &installed).await.err(),
        Some(PluginError::Denied)
    );
    peer.control.unbound_tool.store(false, Ordering::SeqCst);
    probe(&runtime, &owned, &installed)
        .await
        .expect("rejected inventory released session quota");
    assert_eq!(
        peer.control
            .observed
            .lock()
            .expect("observed")
            .iter()
            .filter(|m| *m == "DELETE")
            .count(),
        1
    );
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn mcp_metadata_nul_description_projects_usable_tool_after_enable() {
    let temp = TempRoot::new();
    let owned = owner("nul-description");
    let package = write_mcp_package(&temp.0.join("package"));
    let peer = McpPeer::start(temp.state());
    peer.control.nul_description.store(true, Ordering::SeqCst);
    let runtime = PluginRuntime::with_packages(temp.state(), vec![package]).expect("runtime");
    install(&runtime, &owned).await;
    let installed = configure(&runtime, &owned, json!({"endpoint":peer.endpoint})).await;
    let discovered = probe(&runtime, &owned, &installed)
        .await
        .expect("valid untrusted metadata");
    grant_enable(&runtime, &owned, &installed, &discovered).await;
    let session = runtime
        .session(&owned.0, &owned.1)
        .await
        .expect("model projection");
    let definitions = session.definitions();
    assert_eq!(definitions.len(), 1);
    let mut catalog = ChatToolCatalog::without_opportunity();
    catalog.register_dynamic(definitions).expect("catalog");
    let definition = catalog
        .definitions()
        .into_iter()
        .find(|tool| tool.name.starts_with("plugin_"))
        .expect("MCP tool");
    assert!(!definition.description.contains('\0'));
    let name = definition.name;
    assert_eq!(
        runtime
            .execute_frozen(&session, &name, json!({"query":"synthetic"}))
            .await
            .status(),
        ChatToolStatus::Succeeded
    );
}
