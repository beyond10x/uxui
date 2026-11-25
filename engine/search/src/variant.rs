//! The design space as text.
//!
//! A variant is the base document with at most two edits made in its text, so every other byte
//! stays as written and the checker reads the variant as its author would have written it:
//!
//! - section order: the entries of one page's block sequence `sections:` in another order, each
//!   entry carrying the blank and comment lines that follow it;
//! - theme: the `default` of the top-level `theme:` mapping (flow or block style) set to the
//!   theme's name, or `theme: {default: <name>}` appended when the document has no `theme:`.
//!
//! A page or a `sections:` written in flow style is refused with a message, never guessed at.

/// The section orders of a page with `count` sections: the first `cap` of them in every order
/// ([`permutations`]), the rest after them where they stand. Each order lists section indices.
pub fn orders(count: usize, cap: usize) -> Vec<Vec<usize>> {
    let permuted = count.min(cap);
    permutations(permuted)
        .into_iter()
        .map(|mut order| {
            order.extend(permuted..count);
            order
        })
        .collect()
}

/// Every order of `0..n`, in lexicographic order, from `0, 1, ..., n-1` to its reverse: `n!`
/// orders, and the one empty order for `n = 0`.
pub fn permutations(n: usize) -> Vec<Vec<usize>> {
    let mut current: Vec<usize> = (0..n).collect();
    let mut all = vec![current.clone()];
    loop {
        let Some(head) = (1..current.len())
            .rev()
            .find(|&at| current[at - 1] < current[at])
        else {
            return all;
        };
        let pivot = head - 1;
        let swap = (head..current.len())
            .rev()
            .find(|&at| current[at] > current[pivot])
            .unwrap_or(head);
        current.swap(pivot, swap);
        current[head..].reverse();
        all.push(current.clone());
    }
}

/// One entry of a page's `sections:` as written: the section's name and its lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionText {
    /// The entry's `name`.
    pub name: String,
    /// Its lines, with the blank and comment lines that follow it before the next entry.
    pub text: String,
}

/// A document split around one page's `sections:` entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sections {
    /// The text before the first entry.
    pub before: String,
    /// The entries, in the order written.
    pub items: Vec<SectionText>,
    /// The text after the last entry.
    pub after: String,
}

impl Sections {
    /// Finds the block sequence `sections:` of page `page` under the top-level block mapping
    /// `pages:`.
    pub fn find(text: &str, page: &str) -> Result<Self, String> {
        let lines = lines(text);
        let offset = |index: usize| lines.get(index).map_or(text.len(), |&(start, _)| start);
        let line = |index: usize| lines[index].1;
        let content_line = |index: usize| !is_filler(lines[index].1);
        // The end of the block after line `from`: the first content line no deeper than `parent`.
        let block_end = |from: usize, parent: usize| {
            (from + 1..lines.len())
                .find(|&index| content_line(index) && indent(line(index)) <= parent)
                .unwrap_or(lines.len())
        };

        let pages = (0..lines.len())
            .find(|&index| {
                indent(line(index)) == 0
                    && matches!(key(line(index)), Some(("pages", value)) if is_empty_value(value))
            })
            .ok_or("the document has no block mapping `pages:` at the top level")?;
        let pages_end = block_end(pages, 0);
        let page_indent = (pages + 1..pages_end)
            .find(|&index| content_line(index))
            .map(|index| indent(line(index)))
            .ok_or("`pages:` holds no page")?;
        let (at, value) = (pages + 1..pages_end)
            .filter(|&index| content_line(index) && indent(line(index)) == page_indent)
            .find_map(|index| match key(line(index)) {
                Some((name, value)) if name == page => Some((index, value)),
                _ => None,
            })
            .ok_or_else(|| format!("the document has no page `{page}` under `pages:`"))?;
        if !is_empty_value(value) {
            return Err(format!(
                "page `{page}` is written in flow style; only a block mapping is rewritten"
            ));
        }

        let page_end = block_end(at, page_indent);
        let child_indent = (at + 1..page_end)
            .find(|&index| content_line(index))
            .map(|index| indent(line(index)))
            .ok_or_else(|| format!("page `{page}` is empty"))?;
        let (sections, value) = (at + 1..page_end)
            .filter(|&index| content_line(index) && indent(line(index)) == child_indent)
            .find_map(|index| match key(line(index)) {
                Some(("sections", value)) => Some((index, value)),
                _ => None,
            })
            .ok_or_else(|| format!("page `{page}` writes no `sections:`"))?;
        if !is_empty_value(value) {
            return Err(format!(
                "the sections of page `{page}` are written in flow style; only a block sequence \
                 is rewritten"
            ));
        }

        let first = (sections + 1..page_end)
            .find(|&index| content_line(index))
            .ok_or_else(|| format!("the sections of page `{page}` are empty"))?;
        let entry_indent = indent(line(first));
        if entry_indent < child_indent || !is_entry(line(first)) {
            return Err(format!(
                "the sections of page `{page}` are not a block sequence"
            ));
        }
        let mut starts = Vec::new();
        let mut end = first;
        for index in first..page_end {
            let current = line(index);
            if is_filler(current) {
                continue;
            }
            let depth = indent(current);
            if depth == entry_indent && is_entry(current) {
                starts.push(index);
            } else if depth <= entry_indent {
                break;
            }
            end = index + 1;
        }

        let mut items = Vec::with_capacity(starts.len());
        for (number, &start) in starts.iter().enumerate() {
            let stop = starts.get(number + 1).copied().unwrap_or(end);
            let item = &text[offset(start)..offset(stop)];
            let name = entry_name(item).ok_or_else(|| {
                format!(
                    "entry {} of the sections of page `{page}` has no name",
                    number + 1
                )
            })?;
            items.push(SectionText {
                name,
                text: item.to_owned(),
            });
        }
        Ok(Self {
            before: text[..offset(first)].to_owned(),
            items,
            after: text[offset(end)..].to_owned(),
        })
    }

    /// The sections' names, in the order written.
    pub fn names(&self) -> Vec<&str> {
        self.items.iter().map(|item| item.name.as_str()).collect()
    }

    /// The document with the entries in `order` (indices into [`Sections::items`]).
    pub fn with_order(&self, order: &[usize]) -> String {
        let mut text = self.before.clone();
        for &index in order {
            let item = &self.items[index].text;
            text.push_str(item);
            if !item.ends_with('\n') {
                text.push('\n');
            }
        }
        text.push_str(&self.after);
        text
    }
}

/// The document with `theme` as the default of its top-level `theme:`.
pub fn with_theme(text: &str, theme: &str) -> Result<String, String> {
    let lines = lines(text);
    let found = lines
        .iter()
        .enumerate()
        .find_map(|(index, &(_, line))| match key(line) {
            Some(("theme", value)) if indent(line) == 0 => Some((index, line, value)),
            _ => None,
        });
    let Some((index, line, value)) = found else {
        let mut out = text.to_owned();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("theme: {default: ");
        out.push_str(theme);
        out.push_str("}\n");
        return Ok(out);
    };
    let (at, replacement) = if value.starts_with('{') {
        (index, flow_theme(line, theme))
    } else if is_empty_value(value) {
        block_theme(&lines, index, theme)
    } else {
        return Err(format!(
            "`theme: {value}` is neither a flow nor a block mapping"
        ));
    };
    let (start, replaced) = lines[at];
    let mut out = String::with_capacity(text.len() + replacement.len());
    out.push_str(&text[..start]);
    out.push_str(&replacement);
    out.push_str(&text[start + replaced.len()..]);
    Ok(out)
}

/// The line `theme: {...}` with its `default` set to `theme`.
fn flow_theme(line: &str, theme: &str) -> String {
    let Some(brace) = line.find('{') else {
        return line.to_owned();
    };
    let open = brace + 1;
    let flow = &line[open..];
    for (start, stop) in flow_entries(flow) {
        let entry = &flow[start..stop];
        if matches!(key(entry), Some(("default", _)))
            && let Some(colon) = entry.find(':')
        {
            return format!(
                "{}{}: {theme}{}",
                &line[..open + start],
                &entry[..colon],
                &line[open + stop..]
            );
        }
    }
    if flow.trim_start().starts_with('}') {
        format!("{}default: {theme}{}", &line[..open], flow.trim_start())
    } else {
        format!("{}default: {theme}, {flow}", &line[..open])
    }
}

/// The line to replace in a block `theme:` at line `index`, and its new text: the child
/// `default:` set to `theme`, or the `theme:` line followed by a new `default:` child.
fn block_theme(lines: &[(usize, &str)], index: usize, theme: &str) -> (usize, String) {
    let mut child_indent = None;
    for (at, &(_, line)) in lines.iter().enumerate().skip(index + 1) {
        if is_filler(line) {
            continue;
        }
        let depth = indent(line);
        if depth == 0 {
            break;
        }
        if child_indent.is_none() {
            child_indent = Some(depth);
        }
        if Some(depth) == child_indent && matches!(key(line), Some(("default", _))) {
            let ending = if line.ends_with('\n') { "\n" } else { "" };
            return (at, format!("{}default: {theme}{ending}", " ".repeat(depth)));
        }
    }
    let (_, line) = lines[index];
    let ending = if line.ends_with('\n') { "" } else { "\n" };
    let depth = child_indent.unwrap_or(2);
    (
        index,
        format!("{line}{ending}{}default: {theme}\n", " ".repeat(depth)),
    )
}

/// The name of a sequence entry: `- name: x`, `- {name: x, ...}`, or a `name:` key of the
/// entry's mapping on a later line.
fn entry_name(item: &str) -> Option<String> {
    let mut rows = item.split_inclusive('\n');
    let first = rows.next()?;
    let after_dash = content(first).strip_prefix('-')?;
    let body = after_dash.trim_start();
    let column = indent(first) + 1 + (after_dash.len() - body.len());
    if let Some(flow) = body.strip_prefix('{') {
        return flow_entries(flow).into_iter().find_map(|(start, stop)| {
            match key(&flow[start..stop]) {
                Some(("name", value)) => scalar(value),
                _ => None,
            }
        });
    }
    if let Some(("name", value)) = key(body) {
        return scalar(value);
    }
    rows.filter(|row| !is_filler(row) && indent(row) == column)
        .find_map(|row| match key(row) {
            Some(("name", value)) => scalar(value),
            _ => None,
        })
}

/// The top-level entries of a flow mapping, `flow` starting after its `{`: each entry's byte
/// range in `flow`, up to the closing `}`.
fn flow_entries(flow: &str) -> Vec<(usize, usize)> {
    let mut entries = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (at, character) in flow.char_indices() {
        match character {
            '{' | '[' => depth += 1,
            '}' | ']' => {
                if depth == 0 {
                    entries.push((start, at));
                    return entries;
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                entries.push((start, at));
                start = at + 1;
            }
            _ => {}
        }
    }
    entries.push((start, flow.len()));
    entries
}

/// Each line of `text` with the byte offset it starts at, line endings kept.
fn lines(text: &str) -> Vec<(usize, &str)> {
    let mut start = 0;
    text.split_inclusive('\n')
        .map(|line| {
            let at = start;
            start += line.len();
            (at, line)
        })
        .collect()
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// The line without its indent and its line ending.
fn content(line: &str) -> &str {
    line.trim_start_matches(' ').trim_end_matches(['\r', '\n'])
}

/// A blank line or a comment line.
fn is_filler(line: &str) -> bool {
    let content = content(line).trim();
    content.is_empty() || content.starts_with('#')
}

/// A block sequence entry, `- ...` or a lone `-`.
fn is_entry(line: &str) -> bool {
    let content = content(line);
    content == "-" || content.starts_with("- ")
}

/// The key and the value text of a mapping line `key: value` (or `key:`), the key unquoted.
fn key(line: &str) -> Option<(&str, &str)> {
    let content = content(line);
    if content.starts_with(['-', '#', '{', '[']) {
        return None;
    }
    let mut from = 0;
    while let Some(found) = content[from..].find(':') {
        let at = from + found;
        let rest = &content[at + 1..];
        if rest.is_empty() || rest.starts_with(' ') {
            return Some((unquote(content[..at].trim()), rest.trim()));
        }
        from = at + 1;
    }
    None
}

/// A value that leaves the key's content to the lines below: nothing, or only a comment.
fn is_empty_value(value: &str) -> bool {
    value.is_empty() || value.starts_with('#')
}

/// A plain or quoted scalar without its trailing comment; `None` when empty.
fn scalar(value: &str) -> Option<String> {
    if value.starts_with('#') {
        return None;
    }
    let value = unquote(value.find(" #").map_or(value, |at| &value[..at]).trim());
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn unquote(text: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = text
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        {
            return inner;
        }
    }
    text
}
