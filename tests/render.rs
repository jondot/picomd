//! Tests for `render()`, driven by the fixtures in `tests/fixtures/`. Each case
//! checks for the markup the client libraries (highlight.js, KaTeX, mermaid) and
//! the GitHub styling depend on.

use picomd::render;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn render_fixture(name: &str) -> String {
    render(&fixture(name))
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected to find:\n  {needle}\nin rendered output:\n{haystack}"
    );
}

#[test]
fn headings_get_github_style_anchors() {
    let out = render_fixture("headings.md");
    // header_ids prefixed with "user-content-" to match GitHub anchors.
    assert_contains(&out, "id=\"user-content-hello-world\"");
    assert_contains(&out, "href=\"#hello-world\"");
    assert_contains(&out, "id=\"user-content-sub-section\"");
}

#[test]
fn gfm_table_renders() {
    let out = render_fixture("table.md");
    assert_contains(&out, "<table>");
    assert_contains(&out, "<th>Name</th>");
    assert_contains(&out, "<td>Ada</td>");
}

#[test]
fn task_list_emits_checkboxes() {
    let out = render_fixture("tasklist.md");
    assert_contains(&out, "<input type=\"checkbox\"");
    assert_contains(&out, "checked");
    assert_contains(&out, "disabled");
}

#[test]
fn strikethrough_renders() {
    let out = render_fixture("strikethrough.md");
    assert_contains(&out, "<del>obsolete</del>");
}

#[test]
fn footnotes_render() {
    let out = render_fixture("footnote.md");
    assert_contains(&out, "class=\"footnote-ref\"");
    assert_contains(&out, "<section class=\"footnotes\"");
}

#[test]
fn warning_alert_renders() {
    let out = render_fixture("alert.md");
    assert_contains(&out, "markdown-alert");
    assert_contains(&out, "markdown-alert-warning");
    assert_contains(&out, "markdown-alert-title");
}

#[test]
fn math_inline_and_block_render() {
    let out = render_fixture("math.md");
    // comrak emits <span data-math-style="inline|display"> — client JS targets these.
    assert_contains(&out, "data-math-style=\"inline\"");
    assert_contains(&out, "E = mc^2");
    assert_contains(&out, "data-math-style=\"display\"");
}

#[test]
fn rust_fence_keeps_language_class_after_sanitizing() {
    let out = render_fixture("code_rust.md");
    // ammonia has to keep `class` or highlight.js can't find the block.
    assert_contains(&out, "class=\"language-rust\"");
}

#[test]
fn mermaid_fence_keeps_language_class() {
    let out = render_fixture("mermaid.md");
    assert_contains(&out, "class=\"language-mermaid\"");
    assert_contains(&out, "graph TD");
}

#[test]
fn autolink_becomes_anchor() {
    let out = render_fixture("autolink.md");
    assert_contains(&out, "<a href=\"https://example.com\"");
}

#[test]
fn xss_is_stripped() {
    let out = render_fixture("xss.md");
    // No executable script element.
    assert!(
        !out.contains("<script"),
        "script element survived sanitizing:\n{out}"
    );
    // No javascript: URL.
    assert!(
        !out.to_lowercase().contains("javascript:"),
        "javascript: URL survived sanitizing:\n{out}"
    );
    // No inline event handler.
    assert!(
        !out.to_lowercase().contains("onerror"),
        "onerror handler survived sanitizing:\n{out}"
    );
    // Benign content still rendered.
    assert_contains(&out, "<strong>safe</strong>");
}
