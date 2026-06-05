//! Every asset the page references should resolve through `assets::serve` with no
//! 404s: the links in the HTML template, and the font URLs inside katex.min.css.

use picomd::assets;
use picomd::template;

/// Pull every root-relative `href="/..."` / `src="/..."` out of the HTML shell.
fn template_asset_paths(html: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for attr in ["href=\"", "src=\""] {
        let mut rest = html;
        while let Some(i) = rest.find(attr) {
            rest = &rest[i + attr.len()..];
            if let Some(end) = rest.find('"') {
                let url = &rest[..end];
                if url.starts_with('/') && url != "/" {
                    paths.push(url.to_string());
                }
                rest = &rest[end..];
            } else {
                break;
            }
        }
    }
    paths
}

#[test]
fn every_template_asset_resolves() {
    let html = template::page("<p>hello</p>");
    let paths = template_asset_paths(&html);
    assert!(
        !paths.is_empty(),
        "template referenced no assets — extraction broken?"
    );
    for p in &paths {
        assert!(
            assets::serve(p).is_some(),
            "template asset 404: {p}\n(all referenced: {paths:?})"
        );
    }
    // Sanity: the canonical set the template promises is fully covered.
    for p in assets::TEMPLATE_ASSETS {
        assert!(assets::serve(p).is_some(), "declared asset 404: {p}");
    }
}

#[test]
fn all_katex_fonts_resolve() {
    for name in assets::FONT_FILES {
        let path = format!("/katex/fonts/{name}");
        assert!(assets::serve(&path).is_some(), "embedded font 404: {path}");
    }
}

#[test]
fn katex_css_font_references_all_resolve() {
    // The CSS loads fonts via url(fonts/KaTeX_*.woff2) relative to /katex/. If any
    // referenced woff2 isn't embedded, KaTeX glyphs silently fall back. Assert the
    // woff2 references the CSS actually uses are all served.
    let (css_bytes, _) = assets::serve("/katex/katex.min.css").expect("katex css missing");
    let css = String::from_utf8_lossy(css_bytes);

    let mut checked = 0;
    let mut rest: &str = &css;
    while let Some(i) = rest.find("url(") {
        rest = &rest[i + 4..];
        let end = match rest.find(')') {
            Some(e) => e,
            None => break,
        };
        let raw = rest[..end].trim_matches(|c| c == '\'' || c == '"');
        rest = &rest[end..];
        // Only check the woff2 references (the format we vendored).
        let url = raw.split('?').next().unwrap_or(raw);
        if !url.ends_with(".woff2") {
            continue;
        }
        let file = url.rsplit('/').next().unwrap_or(url);
        let path = format!("/katex/fonts/{file}");
        assert!(
            assets::serve(&path).is_some(),
            "katex.min.css references a font that isn't embedded: {url} -> {path}"
        );
        checked += 1;
    }
    assert!(checked > 0, "no woff2 references found in katex.min.css");
}

#[test]
fn unknown_path_is_404() {
    assert!(assets::serve("/does/not/exist.js").is_none());
    assert!(assets::serve("/katex/fonts/nope.woff2").is_none());
}
