//! Surgical parser for a KMK `main.py`.
//!
//! It locates `keyboard.keymap = [ ... ]`, splits the file into
//! preamble / keymap / postamble, and tokenizes the keymap into layers of
//! key expressions. Comments inside the keymap are stripped; comments elsewhere
//! are preserved as part of preamble/postamble.

use crate::model::{Key, Keymap, Layer};

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: {}", self.0)
    }
}
impl std::error::Error for ParseError {}

/// Parse a KMK `main.py` source into a [`Keymap`].
pub fn parse_main_py(src: &str) -> Result<Keymap, ParseError> {
    let assign = src
        .find("keyboard.keymap")
        .ok_or_else(|| ParseError("could not find `keyboard.keymap`".into()))?;

    // Find the opening `[` of the keymap list after the assignment.
    let open_rel = src[assign..]
        .find('[')
        .ok_or_else(|| ParseError("could not find start of keymap list".into()))?;
    let open = assign + open_rel;

    let close = matching_bracket(src, open)
        .ok_or_else(|| ParseError("unbalanced brackets in keymap list".into()))?;

    let preamble = src[..open].to_string();
    let inner = &src[open + 1..close];
    let postamble = src[close + 1..].to_string();

    let layers = parse_layers(inner)?;
    let aliases = collect_aliases(&preamble);

    Ok(Keymap {
        preamble,
        postamble,
        aliases,
        layers,
    })
}

/// Given the index of an opening `[`, return the index of its matching `]`,
/// honoring `()` nesting and skipping `# ...` comments.
fn matching_bracket(src: &str, open: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    let mut in_comment = false;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if in_comment {
            if c == '\n' {
                in_comment = false;
            }
            i += 1;
            continue;
        }
        match c {
            '#' => in_comment = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Parse the inner text of the keymap list into layers.
fn parse_layers(inner: &str) -> Result<Vec<Layer>, ParseError> {
    let mut layers = Vec::new();
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Advance to the next top-level `[`, skipping commas/whitespace/comments.
        match bytes[i] as char {
            '[' => {
                let close = matching_bracket(inner, i)
                    .ok_or_else(|| ParseError("unbalanced layer brackets".into()))?;
                let layer_src = &inner[i + 1..close];
                layers.push(parse_layer(layer_src));
                i = close + 1;
            }
            '#' => {
                // Comment between layers — skip to newline.
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    Ok(layers)
}

/// Parse one layer's inner text into keys, splitting on top-level commas.
fn parse_layer(src: &str) -> Layer {
    let mut name: Option<String> = None;
    let mut keys = Vec::new();

    let mut token = String::new();
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut chars = src.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '#' => {
                // Comment: capture as layer name if we haven't started a token
                // and don't have a name yet.
                let mut comment = String::new();
                for cc in chars.by_ref() {
                    if cc == '\n' {
                        break;
                    }
                    comment.push(cc);
                }
                let comment = comment.trim();
                if name.is_none() && token.trim().is_empty() && !comment.is_empty() {
                    name = Some(comment.to_string());
                }
            }
            '(' => {
                paren += 1;
                token.push(c);
            }
            ')' => {
                paren -= 1;
                token.push(c);
            }
            '[' => {
                bracket += 1;
                token.push(c);
            }
            ']' => {
                bracket -= 1;
                token.push(c);
            }
            ',' if paren == 0 && bracket == 0 => {
                push_token(&mut keys, &token);
                token.clear();
            }
            _ => token.push(c),
        }
    }
    push_token(&mut keys, &token);

    Layer { name, keys }
}

fn push_token(keys: &mut Vec<Key>, token: &str) {
    let t = token.trim();
    if !t.is_empty() {
        keys.push(Key::new(t));
    }
}

/// Collect top-level alias names from the preamble: `NAME = <expr with KC.>`.
fn collect_aliases(preamble: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in preamble.lines() {
        let line = line.trim_start();
        // Skip indented (inside function/class) and obvious non-assignments.
        if line.starts_with(' ') {
            continue;
        }
        if let Some(eq) = line.find('=') {
            // Guard against ==, <=, >=, != and keyword args.
            let after = line.as_bytes().get(eq + 1).copied();
            let before = if eq > 0 { line.as_bytes()[eq - 1] } else { b' ' };
            if after == Some(b'=') || matches!(before, b'!' | b'<' | b'>' | b'=') {
                continue;
            }
            let name = line[..eq].trim();
            let value = line[eq + 1..].trim();
            let ident_ok = !name.is_empty()
                && name.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false)
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            if ident_ok && value.contains("KC.") {
                out.push(name.to_string());
            }
        }
    }
    out
}
