use napi_derive::napi;

/// Result of an ESQuery match, with UTF-16 offsets for JavaScript consumption.
#[napi(object)]
pub struct NapiMatchResult {
    /// ESTree node type (e.g., "Identifier", "BinaryExpression")
    #[napi(js_name = "type")]
    pub r#type: String,
    /// UTF-16 code unit offset of the start of the matched node
    pub start: u32,
    /// UTF-16 code unit offset of the end of the matched node
    pub end: u32,
    /// Source text of the matched node
    pub text: String,
}

/// Convert a UTF-8 byte offset to a UTF-16 code unit offset.
///
/// JavaScript strings use UTF-16 internally, while the parser produces UTF-8 byte
/// offsets. This conversion is needed so JS consumers get correct offsets.
fn utf8_byte_to_utf16_offset(source: &str, byte_offset: u32) -> u32 {
    let byte_offset = (byte_offset as usize).min(source.len());
    // The parser guarantees byte offsets at UTF-8 char boundaries.
    // floor_char_boundary rounds down as a safety net if that invariant breaks.
    let byte_offset = source.floor_char_boundary(byte_offset);
    source[..byte_offset].encode_utf16().count() as u32
}

/// Query JavaScript/TypeScript source code with an ESQuery selector.
///
/// Returns matched AST nodes with source locations converted to UTF-16 code
/// unit offsets (the standard for JavaScript strings).
///
/// Returns an empty array when the source has syntax errors, the selector is
/// invalid, or no nodes match.
///
/// `source_type` defaults to `"js"` when omitted or when an unrecognized
/// value is provided.
#[napi]
pub fn query(
    source: String,
    selector: String,
    #[napi(ts_arg_type = "'js' | 'jsx' | 'ts' | 'tsx'")] source_type: Option<String>,
) -> Vec<NapiMatchResult> {
    let st = match source_type.as_deref() {
        Some("jsx") => esquery_rs::JsSourceType::Jsx,
        Some("ts") => esquery_rs::JsSourceType::Ts,
        Some("tsx") => esquery_rs::JsSourceType::Tsx,
        _ => esquery_rs::JsSourceType::Js,
    };

    esquery_rs::query(&source, &selector, st)
        .into_iter()
        .map(|r| NapiMatchResult {
            r#type: r.node_type,
            start: utf8_byte_to_utf16_offset(&source, r.start),
            end: utf8_byte_to_utf16_offset(&source, r.end),
            text: r.text,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_offset_ascii() {
        assert_eq!(utf8_byte_to_utf16_offset("hello", 0), 0);
        assert_eq!(utf8_byte_to_utf16_offset("hello", 3), 3);
        assert_eq!(utf8_byte_to_utf16_offset("hello", 5), 5);
    }

    #[test]
    fn utf16_offset_multibyte() {
        // "aあb" — 'a' is 1 byte, 'あ' is 3 bytes (U+3042), 'b' is 1 byte
        // Byte offsets: a=0, あ=1..4, b=4
        // UTF-16 offsets: a=0, あ=1 (1 code unit), b=2
        let s = "aあb";
        assert_eq!(utf8_byte_to_utf16_offset(s, 0), 0);
        assert_eq!(utf8_byte_to_utf16_offset(s, 1), 1);
        assert_eq!(utf8_byte_to_utf16_offset(s, 4), 2);
        assert_eq!(utf8_byte_to_utf16_offset(s, 5), 3);
    }

    #[test]
    fn utf16_offset_surrogate() {
        // U+1F600 (😀): 4 bytes UTF-8, 2 code units UTF-16 (surrogate pair)
        let s = "a\u{1F600}b";
        assert_eq!(utf8_byte_to_utf16_offset(s, 0), 0); // before 'a'
        assert_eq!(utf8_byte_to_utf16_offset(s, 1), 1); // after 'a'
        assert_eq!(utf8_byte_to_utf16_offset(s, 5), 3); // after emoji
        assert_eq!(utf8_byte_to_utf16_offset(s, 6), 4); // after 'b'
    }
}
