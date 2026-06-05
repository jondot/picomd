//! Markdown rendering for picomd.
//!
//! This lives in the library, away from the GUI, so the pipeline can be tested
//! directly. `render` turns Markdown into sanitized HTML, `assets` maps a URL
//! path to embedded bytes, and `template` builds the page shell.

pub mod assets;
pub mod template;

use ammonia::Builder;
use comrak::{Options, markdown_to_html};
use std::collections::HashSet;

/// Render GitHub-flavored Markdown to sanitized HTML.
///
/// comrak runs with unsafe HTML enabled so alerts, math and tables emit real
/// markup, then ammonia sanitizes. The allowlist keeps `class` and the elements
/// KaTeX, mermaid and task lists rely on.
pub fn render(markdown: &str) -> String {
    let html = markdown_to_html(markdown, &comrak_options());
    sanitize(&html)
}

fn comrak_options() -> Options<'static> {
    let mut o = Options::default();

    let e = &mut o.extension;
    e.table = true;
    e.strikethrough = true;
    e.autolink = true;
    e.tasklist = true;
    e.tagfilter = true;
    e.footnotes = true;
    e.superscript = true;
    e.description_lists = true;
    e.multiline_block_quotes = true;
    e.header_id_prefix = Some("user-content-".to_string());
    e.alerts = true;
    e.math_dollars = true;
    e.math_code = true;
    e.shortcodes = true;

    // Needed so alerts/math/tables emit real HTML; ammonia cleans it after.
    o.render.r#unsafe = true;
    // Keep github_pre_lang off. With it on, comrak emits `<pre lang="rust">` and
    // no class; highlight.js and the mermaid selector both want
    // `<code class="language-rust">`, which is the default.

    o
}

/// Sanitize comrak's (intentionally unsafe) HTML while preserving everything the
/// client-side libraries need to function.
fn sanitize(html: &str) -> String {
    let mut b = Builder::default();

    // Elements KaTeX, mermaid, alerts, footnotes and task lists rely on, beyond
    // ammonia's defaults.
    let mut tags: HashSet<&str> = b.clone_tags();
    for t in [
        "input",
        "section",
        "summary",
        "details",
        "del",
        "sup",
        "sub",
        "span",
        "div",
        "svg",
        "path",
        "g",
        "line",
        "rect",
        "circle",
        "ellipse",
        "polygon",
        "polyline",
        "text",
        "tspan",
        "marker",
        "defs",
        "foreignObject",
        "br",
        "math",
        "semantics",
        "annotation",
        "mrow",
        "mi",
        "mo",
        "mn",
        "msup",
        "msub",
        "mfrac",
        "msqrt",
        "mroot",
        "mtable",
        "mtr",
        "mtd",
        "mstyle",
        "mtext",
        "mspace",
        "munder",
        "mover",
        "munderover",
    ] {
        tags.insert(t);
    }
    b.tags(tags);

    // Preserve `class` on the elements that carry styling/behavioral hooks, plus
    // the attributes KaTeX/mermaid/alerts/task-lists emit.
    b.add_generic_attributes(["class", "style", "id"]);
    b.add_tag_attributes("input", ["type", "checked", "disabled"]);
    b.add_tag_attributes("span", ["aria-hidden", "data-math-style"]);
    b.add_tag_attributes("div", ["data-math-style"]);
    b.add_tag_attributes("code", ["data-math-style"]);
    b.add_tag_attributes("ol", ["start"]);
    b.add_tag_attributes("a", ["name"]);
    // SVG/MathML attributes mermaid and KaTeX render.
    b.add_generic_attributes([
        "viewBox",
        "preserveAspectRatio",
        "xmlns",
        "fill",
        "stroke",
        "stroke-width",
        "d",
        "x",
        "y",
        "x1",
        "y1",
        "x2",
        "y2",
        "cx",
        "cy",
        "r",
        "rx",
        "ry",
        "width",
        "height",
        "points",
        "transform",
        "text-anchor",
        "dominant-baseline",
        "stretchy",
        "mathvariant",
        "encoding",
        "aria-hidden",
    ]);

    // Allow data: URLs for images (GitHub does), keep https/http/mailto.
    b.url_schemes(
        ["http", "https", "mailto", "data"]
            .into_iter()
            .collect::<HashSet<_>>(),
    );

    b.clean(html).to_string()
}
