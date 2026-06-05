//! Builds the HTML shell served at `/`.
//!
//! Assets are linked with root-relative paths so they resolve against whatever
//! origin the custom protocol uses on each platform.

/// The static HTML shell. `__BODY__` is replaced with the rendered Markdown.
/// `enhance()` runs the client libraries over a subtree, and `window.rerender`
/// reuses it so a reload re-runs highlighting, math and diagrams rather than just
/// swapping innerHTML.
const SHELL: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="/github-markdown.css">
<link rel="stylesheet" href="/highlight/github.min.css">
<link rel="stylesheet" href="/katex/katex.min.css">
<title>picomd</title>
<style>
  body { margin: 0; background: #ffffff; }
  .markdown-body {
    box-sizing: border-box;
    min-width: 200px;
    max-width: 980px;
    margin: 0 auto;
    padding: 32px 45px 120px;
  }
  /* GitHub-style alert tint (covers older github-markdown-css that predates alerts) */
  .markdown-alert { padding: 8px 16px; margin-bottom: 16px; border-left: .25em solid #d0d7de; color: inherit; }
  .markdown-alert > :first-child { margin-top: 0; }
  .markdown-alert > :last-child { margin-bottom: 0; }
  .markdown-alert-title { display: flex; font-weight: 500; align-items: center; line-height: 1; }
  .markdown-alert-note { border-left-color: #0969da; }
  .markdown-alert-note .markdown-alert-title { color: #0969da; }
  .markdown-alert-tip { border-left-color: #1a7f37; }
  .markdown-alert-tip .markdown-alert-title { color: #1a7f37; }
  .markdown-alert-important { border-left-color: #8250df; }
  .markdown-alert-important .markdown-alert-title { color: #8250df; }
  .markdown-alert-warning { border-left-color: #9a6700; }
  .markdown-alert-warning .markdown-alert-title { color: #9a6700; }
  .markdown-alert-caution { border-left-color: #cf222e; }
  .markdown-alert-caution .markdown-alert-title { color: #cf222e; }
  .mermaid { display: flex; justify-content: center; margin: 16px 0; }
  .mermaid svg { max-width: 100%; height: auto; }
</style>
</head>
<body>
<article id="content" class="markdown-body">__BODY__</article>
<script src="/highlight/highlight.min.js"></script>
<script src="/katex/katex.min.js"></script>
<script src="/mermaid.min.js"></script>
<script>
(function () {
  if (window.mermaid) {
    mermaid.initialize({ startOnLoad: false, theme: "default", securityLevel: "loose" });
  }
  let mermaidSeq = 0;

  function enhance(root) {
    // 1. Promote mermaid code blocks to <div class="mermaid"> BEFORE highlighting,
    //    so hljs never touches them. comrak emits <pre><code class="language-mermaid">.
    root.querySelectorAll("pre code.language-mermaid").forEach(function (code) {
      const div = document.createElement("div");
      div.className = "mermaid";
      div.textContent = code.textContent;
      code.parentElement.replaceWith(div);
    });

    // 2. ```math fenced blocks render as display math.
    root.querySelectorAll("pre code.language-math").forEach(function (code) {
      const holder = document.createElement("div");
      try {
        katex.render(code.textContent, holder, { displayMode: true, throwOnError: false });
        code.parentElement.replaceWith(holder);
      } catch (e) {}
    });

    // 3. Syntax-highlight remaining real code blocks.
    if (window.hljs) {
      root.querySelectorAll("pre code").forEach(function (block) {
        hljs.highlightElement(block);
      });
    }

    // 4. Inline/display math spans: comrak emits <span data-math-style="inline|display">.
    if (window.katex) {
      root.querySelectorAll("span[data-math-style]").forEach(function (span) {
        const display = span.getAttribute("data-math-style") === "display";
        try {
          katex.render(span.textContent, span, { displayMode: display, throwOnError: false });
        } catch (e) {}
      });
    }

    // 5. Render any mermaid diagrams produced in step 1.
    if (window.mermaid) {
      const nodes = root.querySelectorAll(".mermaid");
      if (nodes.length) {
        nodes.forEach(function (n) { n.id = "mmd-" + (mermaidSeq++); });
        try { mermaid.run({ nodes: nodes }); } catch (e) {}
      }
    }
  }

  // The single live-reload entry point Rust calls via evaluate_script.
  window.rerender = function (html) {
    const content = document.getElementById("content");
    content.innerHTML = html;
    enhance(content);
    window.scrollTo(0, 0);
  };

  // Enhance the server-rendered initial body.
  enhance(document.getElementById("content"));
})();
</script>
</body>
</html>
"#;

/// Build the full HTML document with `body` (already-sanitized rendered Markdown)
/// injected into the content container.
pub fn page(body: &str) -> String {
    SHELL.replace("__BODY__", body)
}
