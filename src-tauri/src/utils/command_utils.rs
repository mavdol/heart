pub fn clean_progress_text(text: &str) -> String {
    let mut result = String::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let ch = bytes[i] as char;

        if ch == '\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() && !(bytes[i] as char).is_ascii_alphabetic() {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            if !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            continue;
        }

        if ch.is_ascii_alphanumeric()
            || ch.is_ascii_whitespace()
            || matches!(
                ch,
                '.' | ',' | ':' | '%' | '/' | '-' | '_' | '(' | ')' | 'B' | 'K' | 'M' | 'G'
            )
        {
            result.push(ch);
        }

        i += 1;
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
