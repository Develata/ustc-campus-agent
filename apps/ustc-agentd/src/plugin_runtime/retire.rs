//! Best-effort transport cleanup never changes installation authority or receipts.
use super::{InstallationId, McpClient, RuntimeState};

pub(super) async fn client(mut client: McpClient) {
    // McpClient enforces its request deadline and never retries DELETE or tools/call.
    let _ = client.close().await;
}

pub(super) async fn component(mut probe: super::ProbedComponent) {
    if let Some(transport) = probe.client.take() {
        client(transport).await;
    }
    for (_, mut child) in probe.additional {
        if let Some(transport) = child.client.take() {
            client(transport).await;
        }
    }
}
pub(super) async fn probe(state: &mut RuntimeState, id: &InstallationId) {
    if let Some(probe) = state.probes.remove(id) {
        component(probe).await;
    }
}
