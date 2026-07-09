use super::types::{SessionHandoffPacket, SessionHandoffRequest};
use crate::context::bundle::BundleEntry;
use crate::context::{build_context_bundle, ContextBundleRequest};
use crate::Storage;

pub(super) fn attach_operational_context(
    storage: &Storage,
    request: &SessionHandoffRequest,
    packet: &mut SessionHandoffPacket,
) {
    if !request.include_operational_context {
        return;
    }

    let bundle_request = ContextBundleRequest {
        query: context_query(request),
        workspace: Some(packet.workspace.clone()),
        session_id: packet.session_id.clone(),
        max_results: Some(80),
        section_limit: Some(12),
        include_artifact_pointers: true,
        ..Default::default()
    };

    match storage.with_connection(|conn| build_context_bundle(conn, &bundle_request)) {
        Ok(bundle) => {
            for stale in &bundle.stale_warnings {
                packet.warnings.push(format!(
                    "Stale context: {}: {}",
                    stale.warning.kind, stale.warning.message
                ));
                packet
                    .source_context_event_ids
                    .push(stale.provenance.event_id);
            }
            for entry in bundle
                .recent_decisions
                .iter()
                .chain(bundle.unresolved_blockers.iter())
                .chain(bundle.failures.iter())
            {
                push_context_entry_id(packet, entry);
            }
            packet.source_context_event_ids.sort_unstable();
            packet.source_context_event_ids.dedup();
        }
        Err(err) => packet
            .warnings
            .push(format!("Operational Context bundle failed: {err}")),
    }
}

fn context_query(request: &SessionHandoffRequest) -> Option<String> {
    request
        .current_goal
        .clone()
        .or_else(|| request.summary.clone())
        .or_else(|| Some("session handoff".to_string()))
}

fn push_context_entry_id(packet: &mut SessionHandoffPacket, entry: &BundleEntry) {
    packet
        .source_context_event_ids
        .push(entry.provenance.event_id);
}
