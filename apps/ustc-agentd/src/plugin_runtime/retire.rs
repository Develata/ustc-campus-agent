//! Best-effort transport cleanup never changes installation authority or receipts.
use super::{InstallationId, McpClient, RuntimeState};

pub(super) async fn client(mut client: McpClient) {
    // McpClient enforces its request deadline and never retries DELETE or tools/call.
    let _ = client.close().await;
}

pub(super) async fn probe(state: &mut RuntimeState, id: &InstallationId) {
    // Remove eligibility before cleanup I/O; even a failed DELETE retires locally.
    if let Some(probe) = state.probes.remove(id)
        && let Some(transport) = probe.client
    {
        client(transport).await;
    }
}
