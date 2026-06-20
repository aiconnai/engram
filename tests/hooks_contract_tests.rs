#[test]
fn post_tool_use_has_no_legacy_fake_success_markers() {
    let source = include_str!("../src/hooks/post_tool_use.rs");
    for marker in [
        concat!("Would create ", "memory"),
        concat!("auto", "_memory"),
    ] {
        assert!(
            !source.contains(marker),
            "PostToolUse must not reintroduce the legacy fake-success marker `{marker}`"
        );
    }
}
