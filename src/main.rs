//! The picomd binary: a tao window with a wry webview that previews a Markdown
//! file and reloads it when the file changes.
//!
//! The page is served over a custom protocol (`picomd://`) instead of `with_html`
//! so the vendored assets load from a stable origin on every platform. When the
//! file changes the watcher wakes the event loop, which re-renders and hands the
//! new body to `window.rerender`.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use picomd::{assets, render, template};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;
use wry::http::{Response, header::CONTENT_TYPE};

/// Custom-protocol scheme. The page loads from `picomd://localhost/`; assets use
/// root-relative paths so they resolve regardless of per-platform host mapping.
const SCHEME: &str = "picomd";

fn read_and_render(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(md) => render(&md),
        Err(e) => render(&format!(
            "# picomd\n\nCould not read `{}`:\n\n> {}",
            path.display(),
            e
        )),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: picomd <file.md>");
            std::process::exit(2);
        }
    };
    let path = PathBuf::from(path);
    if !path.exists() {
        eprintln!("picomd: file not found: {}", path.display());
        std::process::exit(1);
    }
    // Canonicalize so watcher path comparisons are reliable across atomic saves.
    let path = std::fs::canonicalize(&path).unwrap_or(path);

    // Shared current body (rendered Markdown). The protocol handler reads it for
    // `/`; the reload path rewrites it before pushing into the live page.
    let body = Arc::new(Mutex::new(read_and_render(&path)));

    let event_loop = EventLoopBuilder::<()>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title(format!(
            "{} — picomd",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("picomd")
        ))
        .with_inner_size(tao::dpi::LogicalSize::new(900.0, 1000.0))
        .build(&event_loop)?;

    // Custom protocol: `/` -> full page, everything else -> a vendored asset.
    let body_for_proto = Arc::clone(&body);
    let builder = WebViewBuilder::new()
        .with_custom_protocol(SCHEME.into(), move |_id, request| {
            let path = request.uri().path().to_string();
            if path == "/" || path.is_empty() {
                let html = template::page(&body_for_proto.lock().unwrap());
                return Response::builder()
                    .header(CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Cow::<'static, [u8]>::Owned(html.into_bytes()))
                    .unwrap();
            }
            match assets::serve(&path) {
                Some((bytes, mime)) => Response::builder()
                    .header(CONTENT_TYPE, mime)
                    .body(Cow::Borrowed(bytes))
                    .unwrap(),
                None => Response::builder()
                    .status(404)
                    .header(CONTENT_TYPE, "text/plain")
                    .body(Cow::Borrowed(b"not found" as &[u8]))
                    .unwrap(),
            }
        })
        .with_url(format!("{SCHEME}://localhost/"));

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let webview = builder.build(&window)?;
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().unwrap();
        builder.build_gtk(vbox)?
    };

    // Watch the PARENT directory (not the file) so atomic-save renames — which
    // replace the inode — keep firing events. Filter back to our target file.
    let watch_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let target = path.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
        move |res: DebounceEventResult| {
            if let Ok(events) = res {
                let hit = events
                    .iter()
                    .any(|e| e.path == target || e.path.file_name() == target.file_name());
                if hit {
                    let _ = proxy.send_event(());
                }
            }
        },
    )?;
    debouncer
        .watcher()
        .watch(&watch_dir, RecursiveMode::NonRecursive)?;

    let render_path = path.clone();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        // Keep the watcher alive for the program's lifetime.
        let _ = &debouncer;
        match event {
            Event::UserEvent(()) => {
                let html = read_and_render(&render_path);
                *body.lock().unwrap() = html.clone();
                let js = format!(
                    "window.rerender({})",
                    serde_json::to_string(&html).unwrap_or_else(|_| "\"\"".into())
                );
                let _ = webview.evaluate_script(&js);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
