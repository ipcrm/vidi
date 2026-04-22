//! Attach stable, slug-based `id` attributes to each heading so intra-doc
//! anchor links (`[Section](#section)`) and the TOC both navigate correctly.
//!
//! Runs before HTML emission. If a heading already has an id (via the
//! `{#custom}` extension syntax), we keep it — otherwise we derive one from
//! the heading's text content. Duplicates are disambiguated with `-2`, `-3`
//! suffixes, matching the rule GitHub uses.

use pulldown_cmark::{CowStr, Event, Tag, TagEnd};
use std::collections::HashSet;

pub fn rewrite(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut out: Vec<Event<'_>> = Vec::with_capacity(events.len());
    let mut used: HashSet<String> = HashSet::new();
    let mut i = 0;

    while i < events.len() {
        if let Event::Start(Tag::Heading {
            level,
            id,
            classes,
            attrs,
        }) = &events[i]
        {
            // Collect text content between Start(Heading) and End(Heading).
            let mut text = String::new();
            let mut j = i + 1;
            while j < events.len() {
                match &events[j] {
                    Event::Text(t) | Event::Code(t) => text.push_str(t.as_ref()),
                    Event::End(TagEnd::Heading(_)) => break,
                    _ => {}
                }
                j += 1;
            }

            let chosen = if let Some(existing) = id.as_ref() {
                let e = existing.to_string();
                uniquify(e, &mut used)
            } else {
                uniquify(slugify(&text), &mut used)
            };

            out.push(Event::Start(Tag::Heading {
                level: *level,
                id: Some(CowStr::Boxed(chosen.into_boxed_str())),
                classes: classes.clone(),
                attrs: attrs.clone(),
            }));
            i += 1;
            continue;
        }
        out.push(events[i].clone());
        i += 1;
    }

    out
}

fn uniquify(base: String, used: &mut HashSet<String>) -> String {
    if base.is_empty() {
        return unique(String::from("heading"), used);
    }
    unique(base, used)
}

fn unique(base: String, used: &mut HashSet<String>) -> String {
    if !used.contains(&base) {
        used.insert(base.clone());
        return base;
    }
    let mut n = 2usize;
    loop {
        let candidate = format!("{base}-{n}");
        if !used.contains(&candidate) {
            used.insert(candidate.clone());
            return candidate;
        }
        n += 1;
    }
}

fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            for l in c.to_lowercase() {
                out.push(l);
            }
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("1.2 — Intro!"), "1-2-intro");
        assert_eq!(slugify("    "), "");
    }

    #[test]
    fn uniquify_appends_suffix() {
        let mut used = HashSet::new();
        assert_eq!(unique("a".into(), &mut used), "a");
        assert_eq!(unique("a".into(), &mut used), "a-2");
        assert_eq!(unique("a".into(), &mut used), "a-3");
    }

    #[test]
    fn empty_base_falls_back_to_heading() {
        let mut used = HashSet::new();
        assert_eq!(uniquify(String::new(), &mut used), "heading");
        assert_eq!(uniquify(String::new(), &mut used), "heading-2");
    }
}
