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
        "img",
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

    let sanitized = Builder::new()
        .tags(allowed_tags)
        .add_tag_attributes("img", &["src", "alt", "width", "height"])
        .add_url_schemes(&["cid"])
        .attribute_filter(|element, attribute, value| {
            if element == "img" && attribute == "src" {
                let lowercase = value.trim().to_ascii_lowercase();
                if !lowercase.starts_with("cid:") {
                    return None;
                }
            }
            Some(value.into())
        })
        .link_rel(Some("noopener noreferrer nofollow"))
        .clean(raw_html)
        .to_string();

    strip_non_cid_image_tags(&sanitized)
}

fn strip_non_cid_image_tags(sanitized_html: &str) -> String {
    let mut output = String::with_capacity(sanitized_html.len());
    let mut cursor = 0;

    while let Some(relative_start) = sanitized_html[cursor..].find("<img") {
        let tag_start = cursor + relative_start;
        output.push_str(&sanitized_html[cursor..tag_start]);

        let remainder = &sanitized_html[tag_start..];
        let Some(tag_end_relative) = remainder.find('>') else {
            break;
        };
        let tag_slice = &remainder[..=tag_end_relative];
        let next_char = remainder.as_bytes().get(4).copied();
        let is_img_element = matches!(next_char, Some(b' ') | Some(b'>') | Some(b'/'));

        if !is_img_element || tag_slice.to_ascii_lowercase().contains("src=\"cid:") {
            output.push_str(tag_slice);
        }

        cursor = tag_start + tag_end_relative + 1;
    }

    output.push_str(&sanitized_html[cursor..]);
    output
}

#[cfg(test)]
mod tests {
    use super::sanitize_email_html;

    #[test]
    fn preserves_cid_images_and_strips_remote_images_and_scripts() {
        let dirty = r#"<p>Hello</p><img src="cid:logo@notifytrap" alt="Logo" width="120" onerror="alert(1)"><img src="https://tracker.example/pixel.png" alt="Track"><a href="javascript:alert(2)">Bad</a><script>evil()</script>"#;
        let clean = sanitize_email_html(dirty);
        assert!(clean.contains(r#"src="cid:logo@notifytrap""#));
        assert!(clean.contains(r#"alt="Logo""#));
        assert!(!clean.contains("tracker.example"));
        assert!(!clean.contains("onerror"));
        assert!(!clean.contains("javascript:"));
        assert!(!clean.contains("<script>"));
    }
}
