use std::collections::HashSet;

use ammonia::Builder;

/// Sanitizes untrusted captured email HTML before previewing inside the sandboxed iframe.
pub fn sanitize_email_html(raw_html: &str) -> String {
    let allowed_tags: HashSet<&str> = [
        "a",
        "abbr",
        "b",
        "blockquote",
        "br",
        "code",
        "div",
        "em",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "hr",
        "i",
        "li",
        "ol",
        "p",
        "pre",
        "span",
        "strong",
        "table",
        "tbody",
        "td",
        "th",
        "thead",
        "tr",
        "u",
        "ul",
    ]
    .into_iter()
    .collect();

    Builder::new()
        .tags(allowed_tags)
        .link_rel(Some("noopener noreferrer nofollow"))
        .clean(raw_html)
        .to_string()
}
