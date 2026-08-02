//! SSH frontend — serves the site as a ratatui TUI over SSH.
//!
//! A plain scrollable page view with a key-hint footer. Each page is reachable
//! by a unique nav key (the first free letter of its nav label; collisions fall
//! through), generated from the discovered pages, so adding a page needs no
//! change here. Rendered by ratatui through a `CrosstermBackend` writing ANSI
//! into a `Vec`, flushed down the SSH channel after every `Terminal::draw`;
//! ratatui buys us scrolling (arrows / space / PageUp-Down) and resize handling.
//! Terminal size comes from the PTY request and is kept current by
//! `window_change` (resize) events. Public access: any auth method is accepted.

use std::net::SocketAddr;
use std::sync::Arc;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{Frame, TerminalOptions, Viewport};
use russh::keys::{PrivateKey, ssh_key};
use russh::server::ChannelOpenHandle;
use russh::server::{Auth, Config, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId, Pty};

use crate::content::Page;

// Clear the screen and home the cursor (erase display, cursor to top-left).
const CLEAR: &[u8] = b"\x1b[2J\x1b[H";

pub async fn serve(addr: String, pages: Arc<Vec<Page>>) -> std::io::Result<()> {
    let config = Arc::new(Config {
        keys: vec![host_key()?],
        ..Config::default()
    });

    let mut server = AppServer { pages };
    server.run_on_address(config, addr).await
}

// Load the SSH host key from $SSH_HOST_KEY (an OpenSSH-format private key) so
// the fingerprint stays stable across deploys. Falls back to an ephemeral key
// when unset — fine for local dev, but a redeploy then changes the fingerprint.
fn host_key() -> std::io::Result<PrivateKey> {
    std::env::var("SSH_HOST_KEY").map_or_else(
        |_| {
            PrivateKey::random(&mut rand::rng(), ssh_key::Algorithm::Ed25519)
                .map_err(std::io::Error::other)
        },
        |pem| PrivateKey::from_openssh(pem).map_err(std::io::Error::other),
    )
}

struct AppServer {
    pages: Arc<Vec<Page>>,
}

impl Server for AppServer {
    type Handler = Conn;

    fn new_client(&mut self, _peer: Option<SocketAddr>) -> Conn {
        Conn {
            pages: self.pages.clone(),
            size: (80, 24),
            term: None,
            app: App::new(self.pages.len()),
        }
    }
}

// The ANSI-emitting backend: ratatui writes escape sequences into a Vec, which
// `Conn::render` drains and sends over the SSH channel after each draw.
type Term = Terminal<CrosstermBackend<Vec<u8>>>;

struct Conn {
    pages: Arc<Vec<Page>>,
    size: (u16, u16),
    term: Option<Term>,
    app: App,
}

// UI state, kept separate from the SSH plumbing so its navigation logic is
// testable without a live session. `content_h`/`content_lines` are recorded on
// each render so key handling can page/clamp scrolling against the real layout.
struct App {
    count: usize,
    selected: usize,
    scroll: u16,
    content_h: u16,
    content_lines: u16,
}

impl App {
    const fn new(count: usize) -> Self {
        Self {
            count,
            selected: 0,
            scroll: 0,
            content_h: 0,
            content_lines: 0,
        }
    }

    // Jump to a page by index (from a nav key), scrolling back to the top.
    fn goto(&mut self, idx: usize) {
        self.selected = idx.min(self.count.saturating_sub(1));
        self.scroll = 0;
    }

    fn scroll_down(&mut self, step: u16) {
        let max = self.content_lines.saturating_sub(self.content_h);
        self.scroll = self.scroll.saturating_add(step).min(max);
    }

    const fn scroll_up(&mut self, step: u16) {
        self.scroll = self.scroll.saturating_sub(step);
    }
}

impl Handler for Conn {
    type Error = russh::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _key: &ssh_key::PublicKey,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        reply: ChannelOpenHandle,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        reply.accept().await;
        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.size = (dim(col_width), dim(row_height));
        session.channel_success(channel)?;
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.size = (dim(col_width), dim(row_height));
        if let Some(term) = self.term.as_mut() {
            let (w, h) = self.size;
            // resize() resets the back buffer, so the next draw is a full redraw;
            // wipe the client screen too so stale cells at the old size are gone.
            term.resize(Rect::new(0, 0, w, h))?;
            term.backend_mut().writer_mut().extend_from_slice(CLEAR);
        }
        self.render(channel, session)?;
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_success(channel)?;
        self.ensure_term()?;
        self.render(channel, session)?;
        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let mut dirty = false;
        let mut i = 0;
        while let Some(&b) = data.get(i) {
            // A page's worth of scroll, for space / PageUp / PageDown.
            let page = self.app.content_h.max(1);
            match b {
                // q, Q, Ctrl-C, Ctrl-D quit.
                b'q' | b'Q' | 3 | 4 => {
                    // Show the cursor again before leaving.
                    session.data(channel, b"\x1b[?25h\r\nBye.\r\n".to_vec())?;
                    session.close(channel)?;
                    return Ok(());
                }
                // Escape sequences: arrows (ESC [ A/B scroll a line) and
                // PageUp/Down (ESC [ 5~/6~ scroll a page).
                // ponytail: only whole, exact-shape sequences within one data()
                // chunk are recognised; a sequence split across TCP reads or a
                // modified variant (ESC [ 5 ; 2 ~) is dropped. Fine interactively;
                // add a carry-over buffer for a partial ESC tail if it bites.
                0x1b if data.get(i.saturating_add(1)) == Some(&b'[') => {
                    match data.get(i.saturating_add(2)) {
                        Some(b'A') => self.app.scroll_up(1),
                        Some(b'B') => self.app.scroll_down(1),
                        Some(b'5') if data.get(i.saturating_add(3)) == Some(&b'~') => {
                            self.app.scroll_up(page);
                            i = i.saturating_add(4);
                            dirty = true;
                            continue;
                        }
                        Some(b'6') if data.get(i.saturating_add(3)) == Some(&b'~') => {
                            self.app.scroll_down(page);
                            i = i.saturating_add(4);
                            dirty = true;
                            continue;
                        }
                        _ => {
                            i = i.saturating_add(1);
                            continue;
                        }
                    }
                    i = i.saturating_add(3);
                    dirty = true;
                    continue;
                }
                // Space pages down; a letter jumps to the page it's the nav key for.
                b' ' => self.app.scroll_down(page),
                _ => {
                    let key = char::from(b).to_ascii_lowercase();
                    if let Some(idx) = page_key_index(&self.pages, key) {
                        self.app.goto(idx);
                        dirty = true;
                    }
                    i = i.saturating_add(1);
                    continue;
                }
            }
            dirty = true;
            i = i.saturating_add(1);
        }
        if dirty {
            self.render(channel, session)?;
        }
        Ok(())
    }
}

impl Conn {
    // Create the terminal on first use, sized to the PTY. A Fixed viewport uses
    // our size directly and never queries the (server-side) tty for dimensions.
    fn ensure_term(&mut self) -> Result<(), russh::Error> {
        if self.term.is_some() {
            return Ok(());
        }
        let (w, h) = self.size;
        let backend = CrosstermBackend::new(Vec::new());
        let mut term = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::new(0, 0, w, h)),
            },
        )?;
        term.hide_cursor()?;
        // Clear the client screen with a raw escape rather than Terminal::clear():
        // its Fixed-viewport path goes through a crossterm call that opens the
        // controlling tty, which fails (ENXIO) on this headless SSH server. The
        // first draw then paints every cell, so a blank baseline is all we need.
        term.backend_mut().writer_mut().extend_from_slice(CLEAR);
        self.term = Some(term);
        Ok(())
    }

    // Draw the UI and flush the accumulated ANSI down the channel.
    fn render(&mut self, channel: ChannelId, session: &mut Session) -> Result<(), russh::Error> {
        let Some(term) = self.term.as_mut() else {
            return Ok(());
        };
        let app = &mut self.app;
        let pages = self.pages.as_ref();
        term.draw(|f| ui(f, app, pages))?;
        let buf = std::mem::take(term.backend_mut().writer_mut());
        if !buf.is_empty() {
            session.data(channel, buf)?;
        }
        Ok(())
    }
}

// Clamp a client-supplied PTY dimension into a sane terminal size. The upper
// bound matters for safety, not just sanity: ratatui eagerly allocates two
// width*height cell buffers, so an unclamped size (up to u16::MAX each) would
// let one connection request a multi-gigabyte allocation and OOM the process.
// MAX_DIM caps a single terminal's buffers at ~2*500*500 cells.
fn dim(v: u32) -> u16 {
    const MAX_DIM: u32 = 500;
    u16::try_from(v.clamp(1, MAX_DIM)).unwrap_or(u16::MAX)
}

// Render the two-pane layout: page menu, scrollable content, key-hint footer.
fn ui(frame: &mut Frame, app: &mut App, pages: &[Page]) {
    let [body, foot] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

    let text = pages
        .get(app.selected)
        .map(|p| render_text(&p.body))
        .unwrap_or_default();
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    let lines = u16::try_from(paragraph.line_count(body.width)).unwrap_or(u16::MAX);

    // Record layout for the next key event and clamp scroll to the content.
    app.content_h = body.height;
    app.content_lines = lines;
    app.scroll = app.scroll.min(lines.saturating_sub(body.height));

    frame.render_widget(paragraph.scroll((app.scroll, 0)), body);
    frame.render_widget(
        Paragraph::new(footer(pages)).style(Style::default().add_modifier(Modifier::DIM)),
        foot,
    );
}

// The nav key for each page: the first letter of its nav label not already
// taken, scanning left to right. Collisions (Colophon and Contact both want
// 'c') fall through to the next free letter, so every page stays reachable.
// 'q' is pre-reserved for quit. None if all its letters are taken.
fn assign_keys(pages: &[Page]) -> Vec<Option<char>> {
    let mut used = vec!['q'];
    pages
        .iter()
        .map(|page| {
            let key = page
                .nav
                .chars()
                .map(|c| c.to_ascii_lowercase())
                .find(|c| c.is_ascii_alphanumeric() && !used.contains(c));
            if let Some(k) = key {
                used.push(k);
            }
            key
        })
        .collect()
}

// Index of the page whose nav key is `key`, if any.
fn page_key_index(pages: &[Page], key: char) -> Option<usize> {
    assign_keys(pages).iter().position(|k| *k == Some(key))
}

// A footer listing every page's key + label, then scroll/quit hints.
fn footer(pages: &[Page]) -> String {
    let mut out = String::new();
    for (page, key) in pages.iter().zip(assign_keys(pages)) {
        if let Some(key) = key {
            out.push('[');
            out.push(key);
            out.push_str("] ");
            out.push_str(&page.nav.to_lowercase());
            out.push_str("  ");
        }
    }
    out.push_str("[space] scroll  [q] quit");
    out
}

// Markdown -> plain text. Drops syntax markers; blocks separated by blank
// lines, list items prefixed with a bullet.
// ponytail: ordered lists render as bullets too; number them if it matters.
fn render_text(markdown: &str) -> String {
    let mut out = String::new();
    for event in Parser::new(markdown) {
        match event {
            Event::Text(t) | Event::Code(t) => out.push_str(&t),
            Event::Start(Tag::Item) => out.push_str("- "),
            Event::SoftBreak | Event::HardBreak | Event::End(TagEnd::Item | TagEnd::List(_)) => {
                out.push('\n');
            }
            Event::End(TagEnd::Paragraph | TagEnd::Heading(_)) => out.push_str("\n\n"),
            _ => {}
        }
    }
    out.trim_end().to_owned()
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::{App, CLEAR, Page, dim, footer, page_key_index, render_text, ui};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn page(nav: &str, body: &str) -> Page {
        Page {
            title: nav.to_owned(),
            nav: nav.to_owned(),
            order: 0,
            path: String::new(),
            body: body.to_owned(),
        }
    }

    #[test]
    fn dim_clamps_hostile_sizes() {
        // A client-controlled huge dimension must be bounded, or ratatui's
        // eager width*height buffer allocation OOMs the process.
        assert_eq!(dim(0), 1, "zero floored to 1 cell");
        assert_eq!(dim(80), 80, "normal size passes through");
        assert_eq!(dim(u32::MAX), 500, "hostile size capped at MAX_DIM");
    }

    #[test]
    fn footer_lists_page_keys_then_hints() {
        let pages = vec![page("Home", ""), page("About", "")];
        assert_eq!(
            footer(&pages),
            "[h] home  [a] about  [space] scroll  [q] quit"
        );
    }

    #[test]
    fn footer_resolves_first_letter_collisions() {
        // Colophon takes 'c'; Contact falls through to its next free letter 'o'.
        let pages = vec![page("Colophon", ""), page("Contact", "")];
        assert_eq!(
            footer(&pages),
            "[c] colophon  [o] contact  [space] scroll  [q] quit"
        );
    }

    #[test]
    fn nav_key_maps_to_its_page_and_goto_clamps() {
        let pages = vec![page("Home", ""), page("About", "")];
        assert_eq!(page_key_index(&pages, 'a'), Some(1), "'a' -> About");
        assert_eq!(page_key_index(&pages, 'z'), None, "unassigned key -> none");

        let mut app = App::new(2);
        app.scroll = 3;
        app.goto(1);
        assert_eq!(app.selected, 1);
        assert_eq!(app.scroll, 0, "jumping to a page resets scroll");
        app.goto(9); // out of range
        assert_eq!(app.selected, 1, "goto clamps to the last page");
    }

    #[test]
    fn scroll_clamps_to_content() {
        let mut app = App::new(2);
        app.content_h = 10;
        app.content_lines = 15; // max scroll = 5
        app.scroll_down(10);
        assert_eq!(app.scroll, 5, "clamped to content, not past the end");
        app.scroll_up(2);
        assert_eq!(app.scroll, 3);
        app.scroll_up(99);
        assert_eq!(app.scroll, 0, "clamped at the top");
    }

    #[test]
    fn render_text_strips_syntax_and_bullets_lists() {
        let out = render_text("# Title\n\nHello\n\n1. one\n2. two");
        assert!(out.contains("Title"), "heading text kept");
        assert!(!out.contains('#'), "heading marker dropped");
        assert!(out.contains("Hello"));
        assert!(
            out.contains("- one") && out.contains("- two"),
            "items bulleted"
        );
    }

    #[test]
    fn ui_renders_without_panicking() {
        let pages = vec![
            page("Home", "# Welcome\n\nHello"),
            page("About", "About me"),
        ];
        // A normal size and a degenerate one: the layout math must not panic
        // (a clippy-denied subtraction underflow would surface here).
        for (w, h) in [(60, 20), (1, 1)] {
            let mut app = App::new(pages.len());
            // TestBackend is infallible, so `match e {}` discharges the Result
            // without an unwrap (clippy forbids unwrap even in tests here).
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap_or_else(|e| match e {});
            term.draw(|f| ui(f, &mut app, &pages))
                .unwrap_or_else(|e| match e {});
        }
    }

    // Regression: the real CrosstermBackend render path (raw CLEAR + hide_cursor
    // + draw over a Fixed viewport) must succeed and emit the page content.
    // Terminal::clear() used to be here and errored with ENXIO on a headless
    // server, killing the session before anything rendered.
    #[test]
    fn crossterm_backend_renders_content() -> Result<(), russh::Error> {
        use ratatui::backend::CrosstermBackend;
        use ratatui::layout::Rect;
        use ratatui::{TerminalOptions, Viewport};

        let pages = vec![page("Home", "# Welcome\n\nHello there")];
        let mut app = App::new(pages.len());
        let mut term = Terminal::with_options(
            CrosstermBackend::new(Vec::new()),
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::new(0, 0, 80, 24)),
            },
        )?;
        term.hide_cursor()?;
        term.backend_mut().writer_mut().extend_from_slice(CLEAR);
        term.draw(|f| ui(f, &mut app, &pages))?;

        let ansi = String::from_utf8_lossy(term.backend_mut().writer_mut());
        assert!(ansi.contains("\x1b[2J"), "clears the client screen");
        assert!(ansi.contains("Welcome"), "page content rendered");
        assert!(ansi.contains("quit"), "footer hint rendered");
        Ok(())
    }
}
