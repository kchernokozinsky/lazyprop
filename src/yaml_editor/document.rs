//! A source-preserving YAML document model.
//!
//! The document keeps the original text verbatim and parses it into a tree of
//! nodes where every scalar remembers the **byte span** of its value token.
//! Edits (manual edit, encrypt, decrypt) replace only that one span, so
//! comments, ordering, blank lines, indentation and unrelated quoting are left
//! untouched. Constructs we cannot edit safely (flow style, block/multiline
//! scalars, anchors/aliases/tags) are parsed for navigation but flagged
//! non-editable rather than silently rewritten.

/// A segment of a node's logical path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSeg {
    Key(String),
    Index(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Mapping,
    Sequence,
    Scalar,
}

/// How a scalar's value is written in the source, which decides whether we can
/// edit it in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarStyle {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    /// Flow (`{}`/`[]`), block (`|`/`>`), tag/anchor/alias, or an empty/null
    /// value — navigable but not editable in place.
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct Node {
    /// Index into [`Document::nodes`]; used only for rendering.
    pub id: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub depth: usize,
    /// Stable logical path, e.g. `database.credentials.password` /
    /// `servers[0].host`. This is what async operations target.
    pub path: Vec<PathSeg>,
    /// Display label: the key name or `[i]`.
    pub label: String,
    pub kind: NodeKind,
    /// Byte span of the scalar value token in [`Document::raw`].
    pub value_span: Option<(usize, usize)>,
    pub style: ScalarStyle,
}

impl Node {
    pub fn is_editable_scalar(&self) -> bool {
        self.kind == NodeKind::Scalar
            && self.value_span.is_some()
            && self.style != ScalarStyle::Unsupported
    }
}

/// Render a path as a dotted/bracketed string (`servers[0].host`).
pub fn path_to_string(path: &[PathSeg]) -> String {
    let mut out = String::new();
    for seg in path {
        match seg {
            PathSeg::Key(k) => {
                if !out.is_empty() {
                    out.push('.');
                }
                out.push_str(k);
            }
            PathSeg::Index(i) => {
                out.push('[');
                out.push_str(&i.to_string());
                out.push(']');
            }
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct Document {
    raw: String,
    nodes: Vec<Node>,
    roots: Vec<usize>,
}

impl Document {
    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn roots(&self) -> &[usize] {
        &self.roots
    }

    pub fn node(&self, id: usize) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Find a node id by its logical path.
    pub fn find_by_path(&self, path: &[PathSeg]) -> Option<usize> {
        self.nodes.iter().find(|n| n.path == path).map(|n| n.id)
    }

    /// The raw source text of a scalar's value (including any quotes).
    pub fn value_source(&self, id: usize) -> Option<&str> {
        let (s, e) = self.nodes.get(id)?.value_span?;
        self.raw.get(s..e)
    }

    /// The logical (unquoted) string value of a scalar node.
    pub fn logical_value(&self, id: usize) -> Option<String> {
        let node = self.nodes.get(id)?;
        let src = self.value_source(id)?;
        Some(scalar_logical_value(src, node.style))
    }

    /// Parse raw YAML text into a source-preserving document.
    pub fn parse(raw: &str) -> Document {
        let mut nodes: Vec<Node> = Vec::new();
        let lines = significant_lines(raw);
        let mut cursor = 0usize;
        let indent = lines.first().map(|l| l.indent).unwrap_or(0);
        let roots = parse_block(raw, &lines, &mut cursor, indent, &[], None, &mut nodes);
        Document {
            raw: raw.to_string(),
            nodes,
            roots,
        }
    }

    /// Validate that `text` is well-formed YAML (used before saving/replacing).
    pub fn validate(text: &str) -> Result<(), String> {
        serde_yaml::from_str::<serde_yaml::Value>(text)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Replace the scalar value at `id` with `new_source` (a valid YAML scalar
    /// token). Returns the new full text, or an error if the node is not an
    /// editable scalar or the result is invalid YAML. Never mutates on failure.
    pub fn replace_scalar_source(&self, id: usize, new_source: &str) -> Result<String, String> {
        let node = self
            .nodes
            .get(id)
            .ok_or_else(|| "node not found".to_string())?;
        if !node.is_editable_scalar() {
            return Err("this value cannot be edited in place".to_string());
        }
        let (s, e) = node.value_span.unwrap();
        let mut text = String::with_capacity(self.raw.len() + new_source.len());
        text.push_str(&self.raw[..s]);
        text.push_str(new_source);
        text.push_str(&self.raw[e..]);
        Document::validate(&text)?;
        Ok(text)
    }
}

// --- encrypted-value wrapper helpers ---------------------------------------

/// Whether `plain` (the logical string value, unquoted) looks like a Mule
/// secure-property wrapper `![...]`.
pub fn is_wrapped(plain: &str) -> bool {
    let t = plain.trim();
    t.starts_with("![") && t.ends_with(']') && t.len() >= 3
}

/// Remove a single `![...]` wrapper, returning the inner ciphertext. If there is
/// no wrapper the input is returned unchanged (a bare ciphertext is accepted).
pub fn unwrap_cipher(plain: &str) -> String {
    let t = plain.trim();
    if is_wrapped(t) {
        t[2..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

/// Wrap ciphertext as `![cipher]`, never producing a double wrapper.
pub fn wrap_cipher(cipher: &str) -> String {
    format!("![{}]", unwrap_cipher(cipher))
}

/// Serialize a logical string value as a YAML scalar token, quoting/escaping
/// only when necessary so plain values stay plain.
pub fn serialize_scalar(value: &str) -> String {
    if needs_double_quoting(value) {
        double_quote(value)
    } else {
        value.to_string()
    }
}

/// Serialize a logical string value as a **double-quoted** YAML scalar token,
/// regardless of whether quoting is strictly required. Used for encrypt/decrypt
/// results so a secure property is always written as a quoted string.
pub fn serialize_scalar_quoted(value: &str) -> String {
    double_quote(value)
}

/// Wrap `value` in double quotes, escaping the characters YAML requires.
fn double_quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn needs_double_quoting(v: &str) -> bool {
    if v.is_empty() {
        return true;
    }
    let first = v.chars().next().unwrap();
    if "!&*?|>%@`\"'#-[]{},".contains(first) {
        return true;
    }
    if v != v.trim() {
        return true;
    }
    if v.contains(": ") || v.ends_with(':') || v.contains(" #") || v.contains(['\n', '\t']) {
        return true;
    }
    let low = v.to_ascii_lowercase();
    if matches!(low.as_str(), "true" | "false" | "null" | "~" | "yes" | "no")
        || v.parse::<f64>().is_ok()
    {
        return true;
    }
    false
}

/// The logical (unquoted, unescaped) value of a scalar source token.
pub fn scalar_logical_value(source: &str, style: ScalarStyle) -> String {
    match style {
        ScalarStyle::Plain | ScalarStyle::Unsupported => source.trim().to_string(),
        ScalarStyle::SingleQuoted => {
            let inner = source.trim();
            let inner = inner
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
                .unwrap_or(inner);
            inner.replace("''", "'")
        }
        ScalarStyle::DoubleQuoted => {
            serde_yaml::from_str::<String>(source.trim()).unwrap_or_else(|_| source.trim().into())
        }
    }
}

// --- block parser -----------------------------------------------------------

struct SigLine<'a> {
    indent: usize,
    /// Byte offset in `raw` where the content (after indentation) starts.
    content_start: usize,
    text: &'a str,
}

/// Collect non-blank, non-comment lines with their indentation and byte offset.
fn significant_lines(raw: &str) -> Vec<SigLine<'_>> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for line in raw.split_inclusive('\n') {
        let trimmed_len = line.trim_end_matches(['\n', '\r']).len();
        let content = &line[..trimmed_len];
        let indent = content.len() - content.trim_start().len();
        let body = content.trim_start();
        if !body.is_empty() && !body.starts_with('#') && body != "---" && body != "..." {
            out.push(SigLine {
                indent,
                content_start: offset + indent,
                text: body,
            });
        }
        offset += line.len();
    }
    out
}

fn is_seq_marker(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

/// Parse sibling entries at column `indent`, returning their node ids.
fn parse_block(
    raw: &str,
    lines: &[SigLine<'_>],
    cursor: &mut usize,
    indent: usize,
    parent_path: &[PathSeg],
    parent: Option<usize>,
    nodes: &mut Vec<Node>,
) -> Vec<usize> {
    let mut ids = Vec::new();
    let mut seq_index = 0usize;
    while *cursor < lines.len() {
        let (line_indent, is_seq) = {
            let l = &lines[*cursor];
            (l.indent, is_seq_marker(l.text))
        };
        if line_indent < indent {
            break;
        }
        if line_indent > indent {
            // Malformed relative to this level; skip to stay panic-free.
            *cursor += 1;
            continue;
        }
        if is_seq {
            let id = parse_seq_item(
                raw,
                lines,
                cursor,
                indent,
                parent_path,
                parent,
                seq_index,
                nodes,
            );
            ids.push(id);
            seq_index += 1;
        } else if let Some(id) =
            parse_map_entry(raw, lines, cursor, indent, parent_path, parent, nodes)
        {
            ids.push(id);
        } else {
            *cursor += 1;
        }
    }
    ids
}

#[allow(clippy::too_many_arguments)]
fn parse_seq_item(
    raw: &str,
    lines: &[SigLine<'_>],
    cursor: &mut usize,
    indent: usize,
    parent_path: &[PathSeg],
    parent: Option<usize>,
    seq_index: usize,
    nodes: &mut Vec<Node>,
) -> usize {
    let (content_start, rest) = {
        let l = &lines[*cursor];
        (l.content_start, &l.text[1..]) // after '-'
    };
    let spaces = rest.len() - rest.trim_start().len();
    let rest_trimmed = rest.trim_start();
    let item_col = indent + 1 + spaces;
    let value_off = content_start + 1 + spaces;

    let mut path = parent_path.to_vec();
    path.push(PathSeg::Index(seq_index));
    let id = push_node(nodes, parent, path.clone(), format!("[{seq_index}]"));

    if rest_trimmed.is_empty() {
        // `-` with a nested block on following lines.
        *cursor += 1;
        if let Some(ci) = lines.get(*cursor).map(|l| l.indent) {
            if ci >= item_col {
                let children = parse_block(raw, lines, cursor, ci, &path, Some(id), nodes);
                finish_container(nodes, id, children);
                return id;
            }
        }
        nodes[id].kind = NodeKind::Scalar;
        nodes[id].style = ScalarStyle::Unsupported;
        return id;
    }

    if is_inline_mapping_start(rest_trimmed) {
        // `- key: ...`: first key sits on the dash line at column `item_col`.
        let mut children = Vec::new();
        if let Some(cid) = parse_map_entry_line(
            raw,
            lines,
            cursor,
            item_col,
            value_off,
            rest_trimmed,
            &path,
            Some(id),
            nodes,
        ) {
            children.push(cid);
        }
        let more = parse_block(raw, lines, cursor, item_col, &path, Some(id), nodes);
        children.extend(more);
        nodes[id].kind = NodeKind::Mapping;
        nodes[id].children = children;
        return id;
    }

    // `- scalar`.
    let (span, style) = scalar_span(value_off, rest_trimmed);
    nodes[id].kind = NodeKind::Scalar;
    nodes[id].value_span = Some(span);
    nodes[id].style = style;
    *cursor += 1;
    id
}

fn parse_map_entry(
    raw: &str,
    lines: &[SigLine<'_>],
    cursor: &mut usize,
    indent: usize,
    parent_path: &[PathSeg],
    parent: Option<usize>,
    nodes: &mut Vec<Node>,
) -> Option<usize> {
    let (content_start, text) = {
        let l = &lines[*cursor];
        (l.content_start, l.text)
    };
    parse_map_entry_line(
        raw,
        lines,
        cursor,
        indent,
        content_start,
        text,
        parent_path,
        parent,
        nodes,
    )
}

/// Parse a `key: ...` entry given its text and byte offset (works for both a
/// real line and the inline first key of a `- key:` sequence item).
#[allow(clippy::too_many_arguments)]
fn parse_map_entry_line(
    raw: &str,
    lines: &[SigLine<'_>],
    cursor: &mut usize,
    indent: usize,
    content_start: usize,
    text: &str,
    parent_path: &[PathSeg],
    parent: Option<usize>,
    nodes: &mut Vec<Node>,
) -> Option<usize> {
    let (key, after_colon, value_off) = split_key(text, content_start)?;
    let mut path = parent_path.to_vec();
    path.push(PathSeg::Key(key.clone()));
    let id = push_node(nodes, parent, path.clone(), key);

    let value = after_colon.trim();
    if value.is_empty() {
        *cursor += 1;
        if let Some(ci) = lines.get(*cursor).map(|l| l.indent) {
            let nested_seq =
                ci == indent && lines.get(*cursor).is_some_and(|l| is_seq_marker(l.text));
            if ci > indent || nested_seq {
                let children = parse_block(raw, lines, cursor, ci, &path, Some(id), nodes);
                finish_container(nodes, id, children);
                return Some(id);
            }
        }
        // Empty scalar (null) — navigable, not editable in place.
        nodes[id].kind = NodeKind::Scalar;
        nodes[id].style = ScalarStyle::Unsupported;
        Some(id)
    } else {
        let (span, style) = scalar_span(value_off, &after_colon);
        nodes[id].kind = NodeKind::Scalar;
        nodes[id].value_span = Some(span);
        nodes[id].style = style;
        *cursor += 1;
        Some(id)
    }
}

fn finish_container(nodes: &mut [Node], id: usize, children: Vec<usize>) {
    let is_seq = children
        .iter()
        .any(|&c| matches!(nodes[c].path.last(), Some(PathSeg::Index(_))));
    nodes[id].kind = if is_seq {
        NodeKind::Sequence
    } else {
        NodeKind::Mapping
    };
    nodes[id].children = children;
}

fn push_node(
    nodes: &mut Vec<Node>,
    parent: Option<usize>,
    path: Vec<PathSeg>,
    label: String,
) -> usize {
    let id = nodes.len();
    let depth = path.len().saturating_sub(1);
    nodes.push(Node {
        id,
        parent,
        children: Vec::new(),
        depth,
        path,
        label,
        kind: NodeKind::Scalar,
        value_span: None,
        style: ScalarStyle::Plain,
    });
    id
}

/// Split `key: value` into (key, value-part-with-leading-space, byte offset of
/// the value). Returns None if there is no top-level `:` separator.
fn split_key(text: &str, content_start: usize) -> Option<(String, String, usize)> {
    let bytes = text.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b':' if !in_single && !in_double => {
                let after = &text[i + 1..];
                if after.is_empty() || after.starts_with(' ') {
                    let key = text[..i].trim().trim_matches(['\'', '"']).to_string();
                    return Some((key, after.to_string(), content_start + i + 1));
                }
            }
            b'#' if !in_single && !in_double => break,
            _ => {}
        }
    }
    None
}

fn is_inline_mapping_start(s: &str) -> bool {
    if s.starts_with(['{', '[', '"', '\'']) {
        return false;
    }
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b':' && (i + 1 == bytes.len() || bytes[i + 1] == b' ') {
            return true;
        }
        if b == b'#' {
            break;
        }
    }
    false
}

/// Byte span and style of a scalar value, given the value part (with leading
/// spaces) and the byte offset where that part starts.
fn scalar_span(part_start: usize, part: &str) -> ((usize, usize), ScalarStyle) {
    let leading = part.len() - part.trim_start().len();
    let start = part_start + leading;
    let body = part.trim_start();

    if body.starts_with('"') {
        if let Some(end_rel) = find_double_quote_end(body) {
            return ((start, start + end_rel), ScalarStyle::DoubleQuoted);
        }
    } else if body.starts_with('\'') {
        if let Some(end_rel) = find_single_quote_end(body) {
            return ((start, start + end_rel), ScalarStyle::SingleQuoted);
        }
    } else if body.starts_with(['{', '[', '|', '>', '&', '*', '!']) {
        return (
            (start, start + body.trim_end().len()),
            ScalarStyle::Unsupported,
        );
    }

    // Plain scalar: ends before a trailing " #" comment.
    let end_rel = body.find(" #").unwrap_or(body.len());
    let value_len = body[..end_rel].trim_end().len();
    ((start, start + value_len), ScalarStyle::Plain)
}

fn find_double_quote_end(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

fn find_single_quote_end(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 1;
    while i < bytes.len() {
        if bytes[i] == b'\'' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                i += 2;
            } else {
                return Some(i + 1);
            }
        } else {
            i += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
database:
  host: localhost
  credentials:
    username: admin
    password: secret
servers:
  - host: server-one
    port: 8081
";

    fn nid(doc: &Document, path: &str) -> usize {
        doc.nodes()
            .iter()
            .find(|n| path_to_string(&n.path) == path)
            .unwrap_or_else(|| panic!("path not found: {path}"))
            .id
    }

    #[test]
    fn parses_nested_mappings() {
        let doc = Document::parse(SAMPLE);
        assert_eq!(
            doc.node(nid(&doc, "database")).unwrap().kind,
            NodeKind::Mapping
        );
        assert_eq!(
            doc.node(nid(&doc, "database.credentials")).unwrap().kind,
            NodeKind::Mapping
        );
        let pw = nid(&doc, "database.credentials.password");
        assert_eq!(doc.node(pw).unwrap().kind, NodeKind::Scalar);
        assert_eq!(doc.value_source(pw), Some("secret"));
    }

    #[test]
    fn parses_sequences_and_indexed_paths() {
        let doc = Document::parse(SAMPLE);
        assert_eq!(
            doc.node(nid(&doc, "servers")).unwrap().kind,
            NodeKind::Sequence
        );
        assert_eq!(
            doc.value_source(nid(&doc, "servers[0].host")),
            Some("server-one")
        );
        assert_eq!(doc.value_source(nid(&doc, "servers[0].port")), Some("8081"));
    }

    #[test]
    fn replace_plain_scalar_preserves_everything_else() {
        let doc = Document::parse(SAMPLE);
        let out = doc
            .replace_scalar_source(nid(&doc, "database.credentials.password"), "\"![CIPHER]\"")
            .unwrap();
        assert!(out.contains("password: \"![CIPHER]\""));
        assert!(out.contains("host: localhost"));
        assert!(out.contains("port: 8081"));
        let changed = SAMPLE
            .lines()
            .zip(out.lines())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(changed, 1);
    }

    #[test]
    fn replace_quoted_scalar() {
        let doc = Document::parse("key: \"old value\"\nother: 1\n");
        let k = nid(&doc, "key");
        assert_eq!(doc.value_source(k), Some("\"old value\""));
        assert_eq!(
            doc.replace_scalar_source(k, "new").unwrap(),
            "key: new\nother: 1\n"
        );
    }

    #[test]
    fn replace_value_with_special_chars_is_quoted() {
        let doc = Document::parse("greeting: hello\n");
        let out = doc
            .replace_scalar_source(nid(&doc, "greeting"), &serialize_scalar("a: b # x"))
            .unwrap();
        assert!(out.starts_with("greeting: \""));
        Document::validate(&out).unwrap();
    }

    #[test]
    fn detect_and_unwrap_wrapper() {
        assert!(is_wrapped("![abc]"));
        assert!(!is_wrapped("abc"));
        assert_eq!(unwrap_cipher("![abc]"), "abc");
        assert_eq!(unwrap_cipher("abc"), "abc");
    }

    #[test]
    fn wrap_never_double_wraps() {
        assert_eq!(wrap_cipher("abc"), "![abc]");
        assert_eq!(wrap_cipher("![abc]"), "![abc]");
    }

    #[test]
    fn preserves_comments_and_blank_lines() {
        let src = "# top comment\ndb:\n  pass: secret  # inline\n\nother: 1\n";
        let doc = Document::parse(src);
        let out = doc
            .replace_scalar_source(nid(&doc, "db.pass"), "x")
            .unwrap();
        assert!(out.contains("# top comment"));
        assert!(out.contains("pass: x  # inline"));
        assert!(out.contains("\n\nother: 1"));
    }

    #[test]
    fn invalid_replacement_is_rejected_without_mutating() {
        let doc = Document::parse("a: 1\n");
        assert!(doc.replace_scalar_source(nid(&doc, "a"), ": : :").is_err());
        assert_eq!(doc.raw(), "a: 1\n");
    }

    #[test]
    fn logical_value_of_quoted() {
        assert_eq!(
            scalar_logical_value("\"![x]\"", ScalarStyle::DoubleQuoted),
            "![x]"
        );
        assert_eq!(
            scalar_logical_value("'it''s'", ScalarStyle::SingleQuoted),
            "it's"
        );
        assert_eq!(scalar_logical_value("plain", ScalarStyle::Plain), "plain");
    }

    #[test]
    fn numbers_and_bools_get_quoted_when_serialized() {
        assert_eq!(serialize_scalar("8081"), "\"8081\"");
        assert_eq!(serialize_scalar("true"), "\"true\"");
        assert_eq!(serialize_scalar("plainword"), "plainword");
    }

    #[test]
    fn quoted_serializer_always_quotes() {
        // Even a plain-safe value is quoted, and quotes are escaped.
        assert_eq!(serialize_scalar_quoted("secret"), "\"secret\"");
        assert_eq!(serialize_scalar_quoted("![CIPHER]"), "\"![CIPHER]\"");
        assert_eq!(serialize_scalar_quoted("a\"b"), "\"a\\\"b\"");
    }
}
