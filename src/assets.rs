//! The vendored assets, served over the wry custom protocol.
//!
//! It's a plain lookup with no GUI dependency, so the tests can call [`serve`]
//! directly. Every file is embedded with `include_bytes!`, which is what keeps
//! the app offline.

/// Resolve a URL path (e.g. `/katex/katex.min.css`) to its embedded bytes and
/// MIME type. Returns `None` for unknown paths (the caller responds 404).
///
/// The leading slash is optional and any query string must already be stripped.
pub fn serve(path: &str) -> Option<(&'static [u8], &'static str)> {
    let p = path.trim_start_matches('/');

    // KaTeX fonts: same MIME, indexed by file name.
    if let Some(name) = p.strip_prefix("katex/fonts/") {
        return font(name).map(|b| (b, "font/woff2"));
    }

    let r: (&'static [u8], &'static str) = match p {
        "github-markdown.css" => (
            include_bytes!("../assets/github-markdown.css"),
            "text/css; charset=utf-8",
        ),
        "highlight/github.min.css" => (
            include_bytes!("../assets/highlight/github.min.css"),
            "text/css; charset=utf-8",
        ),
        "highlight/highlight.min.js" => (
            include_bytes!("../assets/highlight/highlight.min.js"),
            "text/javascript; charset=utf-8",
        ),
        "katex/katex.min.css" => (
            include_bytes!("../assets/katex/katex.min.css"),
            "text/css; charset=utf-8",
        ),
        "katex/katex.min.js" => (
            include_bytes!("../assets/katex/katex.min.js"),
            "text/javascript; charset=utf-8",
        ),
        "mermaid.min.js" => (
            include_bytes!("../assets/mermaid.min.js"),
            "text/javascript; charset=utf-8",
        ),
        _ => return None,
    };
    Some(r)
}

/// All asset paths the HTML template references (used by the no-404 test).
pub const TEMPLATE_ASSETS: &[&str] = &[
    "/github-markdown.css",
    "/highlight/github.min.css",
    "/highlight/highlight.min.js",
    "/katex/katex.min.css",
    "/katex/katex.min.js",
    "/mermaid.min.js",
];

macro_rules! fonts {
    ($($name:literal),* $(,)?) => {
        fn font(name: &str) -> Option<&'static [u8]> {
            match name {
                $($name => Some(include_bytes!(concat!("../assets/katex/fonts/", $name))),)*
                _ => None,
            }
        }
        /// Every embedded KaTeX font file name (used by the no-404 test).
        pub const FONT_FILES: &[&str] = &[$($name),*];
    };
}

fonts!(
    "KaTeX_AMS-Regular.woff2",
    "KaTeX_Caligraphic-Bold.woff2",
    "KaTeX_Caligraphic-Regular.woff2",
    "KaTeX_Fraktur-Bold.woff2",
    "KaTeX_Fraktur-Regular.woff2",
    "KaTeX_Main-Bold.woff2",
    "KaTeX_Main-BoldItalic.woff2",
    "KaTeX_Main-Italic.woff2",
    "KaTeX_Main-Regular.woff2",
    "KaTeX_Math-BoldItalic.woff2",
    "KaTeX_Math-Italic.woff2",
    "KaTeX_SansSerif-Bold.woff2",
    "KaTeX_SansSerif-Italic.woff2",
    "KaTeX_SansSerif-Regular.woff2",
    "KaTeX_Script-Regular.woff2",
    "KaTeX_Size1-Regular.woff2",
    "KaTeX_Size2-Regular.woff2",
    "KaTeX_Size3-Regular.woff2",
    "KaTeX_Size4-Regular.woff2",
    "KaTeX_Typewriter-Regular.woff2",
);
