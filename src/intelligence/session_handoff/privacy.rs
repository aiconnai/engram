pub(super) fn strip_private_content(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(start) = remaining.find("<private>") {
        result.push_str(&remaining[..start]);
        if let Some(end_offset) = remaining[start..].find("</private>") {
            remaining = &remaining[start + end_offset + "</private>".len()..];
        } else {
            return result;
        }
    }
    result.push_str(remaining);
    result
}
