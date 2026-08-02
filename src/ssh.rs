//! SSH frontend — serves the site as a ratatui TUI over SSH.
//!
//! A two-pane TUI: a page menu (left) and the selected page's Markdown rendered
//! to text (right, scrollable). Menu and content derive from the discovered
//! pages, so adding a page needs no change here. Rendered by ratatui through a
//! `CrosstermBackend` writing ANSI into a `Vec`, which is flushed down the SSH
//! channel after every `Terminal::draw`. Terminal size comes from the PTY
//! request and is kept current by `window_change` (resize) events.
//! Public access: any auth method is accepted.

use std::net::SocketAddr;
use std::sync::Arc;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};
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

    fn select_next(&mut self) {
        let max = self.count.saturating_sub(1);
        self.selected = self.selected.saturating_add(1).min(max);
        self.scroll = 0;
    }

    const fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.scroll = 0;
    }

    fn scroll_page_down(&mut self) {
        let max = self.content_lines.saturating_sub(self.content_h);
        self.scroll = self.scroll.saturating_add(self.content_h.max(1)).min(max);
    }

    fn scroll_page_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(self.content_h.max(1));
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
            match b {
                // q, Q, Ctrl-C, Ctrl-D quit.
                b'q' | b'Q' | 3 | 4 => {
                    // Show the cursor again before leaving.
                    session.data(channel, b"\x1b[?25h\r\nBye.\r\n".to_vec())?;
                    session.close(channel)?;
                    return Ok(());
                }
                // Escape sequences: arrows (ESC [ A/B) and PageUp/Down (ESC [ 5~/6~).
                // ponytail: only whole, exact-shape sequences within one data()
                // chunk are recognised; a sequence split across TCP reads or a
                // modified variant (ESC [ 5 ; 2 ~) is dropped. Fine interactively;
                // add a carry-over buffer for a partial ESC tail if it bites.
                0x1b if data.get(i.saturating_add(1)) == Some(&b'[') => {
                    match data.get(i.saturating_add(2)) {
                        Some(b'A') => self.app.select_prev(),
                        Some(b'B') => self.app.select_next(),
                        Some(b'5') if data.get(i.saturating_add(3)) == Some(&b'~') => {
                            self.app.scroll_page_up();
                            i = i.saturating_add(4);
                            dirty = true;
                            continue;
                        }
                        Some(b'6') if data.get(i.saturating_add(3)) == Some(&b'~') => {
                            self.app.scroll_page_down();
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
                b'k' => self.app.select_prev(),
                b'j' => self.app.select_next(),
                b' ' | b'f' => self.app.scroll_page_down(),
                b'b' => self.app.scroll_page_up(),
                _ => {
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
    let [menu_area, content_area] =
        Layout::horizontal([Constraint::Length(20), Constraint::Min(1)]).areas(body);

    let items: Vec<ListItem> = pages.iter().map(|p| ListItem::new(p.nav.clone())).collect();
    let menu = List::new(items)
        .block(Block::bordered().title("thombruce.com"))
        .highlight_symbol("> ")
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut menu_state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(menu, menu_area, &mut menu_state);

    let page = pages.get(app.selected);
    let title = page.map(|p| p.title.clone()).unwrap_or_default();
    let text = page.map(|p| render_text(&p.body)).unwrap_or_default();

    let inner_w = content_area.width.saturating_sub(2);
    let inner_h = content_area.height.saturating_sub(2);
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    let lines = u16::try_from(paragraph.line_count(inner_w)).unwrap_or(u16::MAX);

    // Record layout for the next key event and clamp scroll to the content.
    app.content_h = inner_h;
    app.content_lines = lines;
    app.scroll = app.scroll.min(lines.saturating_sub(inner_h));

    frame.render_widget(
        paragraph
            .scroll((app.scroll, 0))
            .block(Block::bordered().title(title)),
        content_area,
    );

    frame.render_widget(
        Paragraph::new("^/v: pages   space/b: scroll   q: quit")
            .style(Style::default().add_modifier(Modifier::DIM)),
        foot,
    );
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
    use super::{App, CLEAR, Page, dim, render_text, ui};
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
    fn select_clamps_at_both_ends() {
        let mut app = App::new(2);
        app.select_prev(); // already at top, stays at 0
        assert_eq!(app.selected, 0);
        app.select_next();
        app.select_next(); // only 2 pages, clamps at index 1
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn scroll_clamps_to_content_and_resets_on_page_change() {
        let mut app = App::new(2);
        app.content_h = 10;
        app.content_lines = 15; // max scroll = 5
        app.scroll_page_down();
        assert_eq!(app.scroll, 5, "clamped to content, not a full page (10)");
        app.select_next();
        assert_eq!(app.scroll, 0, "changing page resets scroll");
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
        assert!(ansi.contains("thombruce.com"), "menu title rendered");
        assert!(ansi.contains("Welcome"), "page content rendered");
        assert!(ansi.contains("quit"), "footer hint rendered");
        Ok(())
    }
}
