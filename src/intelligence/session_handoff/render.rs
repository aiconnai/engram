use super::privacy::strip_private_content;
use super::types::{HandoffItem, SessionHandoffPacket};

pub(super) fn render_copy_block(packet: &SessionHandoffPacket) -> String {
    let mut output = String::new();
    output.push_str("# Continue this work in a new AI session\n\n");
    output.push_str("## Essential context\n");
    output.push_str(&format!("Workspace: {}\n", packet.workspace));
    if let Some(session_id) = &packet.session_id {
        output.push_str(&format!("Session: {session_id}\n"));
    }
    output.push_str(&format!("Summary: {}\n\n", packet.summary));

    output.push_str("## Current goal\n");
    output.push_str(
        packet
            .current_goal
            .as_deref()
            .unwrap_or("No current goal captured."),
    );
    output.push_str("\n\n## Decisions\n");
    push_items(&mut output, &packet.decisions);
    output.push_str("\n## Verification\n");
    push_items(&mut output, &packet.verification);
    push_string_list(&mut output, "Tests not run", &packet.tests_not_run);

    output.push_str("\n## Risks and blockers\n");
    push_items(&mut output, &packet.risks);
    push_items(&mut output, &packet.blockers);

    output.push_str("\n## Next steps\n");
    if packet.next_steps.is_empty() {
        output.push_str("- No next steps captured.\n");
    } else {
        for (index, step) in packet.next_steps.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", index + 1, strip_private_content(step)));
        }
    }

    output.push_str("\n## Source references\n");
    output.push_str(&format!("- Memory IDs: {:?}\n", packet.source_memory_ids));
    output.push_str(&format!(
        "- Context event IDs: {:?}\n",
        packet.source_context_event_ids
    ));
    output.push_str(&format!("- Files: {:?}\n", packet.files_touched));
    push_string_list(&mut output, "Warnings", &packet.warnings);
    output
}

fn push_items(output: &mut String, items: &[HandoffItem]) {
    if items.is_empty() {
        output.push_str("- None captured.\n");
        return;
    }

    for item in items {
        output.push_str(&format!("- {}\n", item.title));
    }
}

fn push_string_list(output: &mut String, title: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }

    output.push('\n');
    output.push_str(title);
    output.push_str(":\n");
    for item in items {
        output.push_str(&format!("- {}\n", strip_private_content(item)));
    }
}
