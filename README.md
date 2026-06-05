# picomd

A small, offline Markdown previewer for the desktop. It opens a `.md` file given
on the command line, renders it in a native window, and reloads when the file
changes on disk. Rendering aims to match GitHub. There is no editor.

- Markdown: [comrak](https://crates.io/crates/comrak) (GFM + GitHub extensions),
  sanitized with [ammonia](https://crates.io/crates/ammonia).
- Window: [tao](https://crates.io/crates/tao) + [wry](https://crates.io/crates/wry),
  with assets served over a custom protocol.
- highlight.js, KaTeX, mermaid and github-markdown-css are vendored into the
  binary, so there are no CDN calls and it works offline.

## Install

curl installer (macOS / Linux):

```sh
curl -fsSL https://raw.githubusercontent.com/jondot/picomd/main/install.sh | sh
```

npm (macOS / Linux):

```sh
npm install -g @jondot/picomd
```

cargo (needs a Rust toolchain and the [GUI build deps](#platforms)):

```sh
cargo install picomd
```

Then:

```sh
picomd path/to/file.md
```

## Build & run

```sh
cargo build --release
./target/release/picomd path/to/file.md
```

```sh
cargo test
```

The release binary is around 5.3 MB stripped. Most of that is the vendored KaTeX
and mermaid bundles (mermaid alone is ~2.6 MB), which is the price of working
offline.

## How it works

```
src/lib.rs       render(markdown) -> sanitized HTML
src/assets.rs    serve(path) -> (bytes, mime)
src/template.rs  page(body) -> HTML shell + client JS
src/main.rs      window, webview and the file watcher
```

`render()` has no GUI dependencies, so the whole Markdown pipeline is tested
against the fixtures in `tests/fixtures/`. Assets are pulled in with
`include_bytes!` and served over the `picomd://` protocol; the HTML references
them with root-relative paths so they resolve the same way on every platform.

Live reload calls one JS function, `window.rerender(html)`. It swaps the article's
`innerHTML` and then re-runs highlight.js, KaTeX and `mermaid.run()` over the new
nodes. Swapping `innerHTML` on its own does not re-trigger those libraries, so a
code block or diagram added in an edit would otherwise stay unrendered.

A couple of details on the pipeline:

- comrak runs with `render.unsafe = true` so alerts, math and tables emit real
  HTML, then ammonia cleans it up. The sanitizer keeps `class` on `pre`, `code`,
  `span` and `div`, allows the task-list checkbox attributes, and keeps the
  SVG/MathML elements that KaTeX and mermaid produce.
- `github_pre_lang` is left off so fenced code stays `<code class="language-*">`
  rather than `<pre lang="...">`; highlight.js needs the class. Mermaid blocks
  arrive as `language-mermaid` and are moved into a `<div class="mermaid">` before
  highlighting so highlight.js never touches them.

## Vendored assets

| Asset | Version |
|-------|---------|
| KaTeX (js + css + 20 woff2 fonts) | 0.16.22 |
| mermaid | 11.6.0 |
| highlight.js (browser bundle) + GitHub theme | 11.11.1 |
| github-markdown-css (light) | 5.8.1 |

Only the woff2 KaTeX fonts are shipped. WKWebView, WebView2 and webkit2gtk all
support woff2, and dropping the other formats keeps the binary smaller.

## Tests

`cargo test` runs the render and asset checks (no GUI needed):

- `tests/render.rs` covers headings and anchors, tables, task lists,
  strikethrough, footnotes, alerts, inline and block math, the `language-rust`
  class surviving sanitizing, mermaid blocks, autolinks, and an XSS case that
  should be stripped.
- `tests/assets.rs` checks that every asset URL the template references resolves,
  that all 20 KaTeX fonts resolve, that every woff2 referenced inside
  `katex.min.css` is embedded, and that unknown paths 404.

The window can't run in CI, so for GUI changes I check by hand that a file opens
and matches GitHub, that an edit shows up within ~200 ms, and that a code block,
math expression or diagram added in that edit renders after saving.

## Platforms

- macOS (Intel and Apple silicon, WKWebView). No extra dependencies.
- Linux x86_64 (webkit2gtk). Building from source needs the GUI dev libraries:

  ```sh
  sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev
  ```

There are Windows code paths (WebView2), but Windows and arm64 Linux are not
built or shipped.

The watcher follows the file's parent directory rather than the inode, so it keeps
working with editors that save by writing a temp file and renaming over the
original.

## Releasing

Releases are tag-driven; see [RELEASING.md](RELEASING.md). `./scripts/release.sh
X.Y.Z` bumps `Cargo.toml`, tags `vX.Y.Z` and pushes. The workflow then builds the
targets, signs them with cosign and publishes a GitHub Release. npm and crates.io
publishing are opt-in (the `PUBLISH_NPM` and `PUBLISH_CRATES` repo variables).

## Limitations

- Preview only: no editing, tabs, multiple files, export or settings.
- Only woff2 KaTeX fonts are shipped, so an environment without woff2 support
  would lose math glyphs. None of the target webviews are affected.
- mermaid runs with `securityLevel: "loose"` so diagrams render fully. The input
  is your own local file, but diagram labels are not sandboxed.
- First load parses the ~2.6 MB mermaid bundle, so initial paint can lag on old
  hardware. Reloads reuse the already-loaded libraries.

## Examples

`cargo run --example page -- file.md` prints the full HTML page and `cargo run
--example body -- file.md` prints just the rendered body.
