//! SSH frontend — serves the site as a ratatui TUI over SSH.
//!
//! Three screens, each a scrollable text view with a key-hint footer:
//! - **Page** — a static page, reached by its nav key (first free letter of its
//!   label; collisions fall through). `Blog` is one of these nav targets.
//! - `BlogIndex` — a paginated post list (10/page); digits `1`-`9`/`0` open an
//!   entry, `>`/`<` page, `h` home.
//! - **Post** — a single post; `b` back to the index.
//!
//! Nav and lists derive from the discovered content, so adding a page or post
//! needs no change here. Rendered by ratatui through a `CrosstermBackend`
//! writing ANSI into a `Vec`, flushed down the SSH channel after every
//! `Terminal::draw`; ratatui buys us scrolling (arrows / space / PageUp-Down)
//! and resize handling. Terminal size comes from the PTY request and is kept
//! current by `window_change` (resize) events. Public access: any auth accepted.

use std::net::SocketAddr;
use std::sync::Arc;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{Frame, TerminalOptions, Viewport};
use russh::keys::{PrivateKey, ssh_key};
use russh::server::ChannelOpenHandle;
use russh::server::{Auth, Config, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId, Pty};

use crate::content::{Content, Page, Post};

// Clear the screen and home the cursor (erase display, cursor to top-left).
const CLEAR: &[u8] = b"\x1b[2J\x1b[H";

pub async fn serve(addr: String, content: Arc<Content>) -> std::io::Result<()> {
    let config = Arc::new(Config {
        keys: vec![host_key()?],
        ..Config::default()
    });

    let mut server = AppServer { content };
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
    content: Arc<Content>,
}

impl Server for AppServer {
    type Handler = Conn;

    fn new_client(&mut self, _peer: Option<SocketAddr>) -> Conn {
        Conn {
            content: self.content.clone(),
            size: (80, 24),
            term: None,
            app: App::new(),
        }
    }
}

// The ANSI-emitting backend: ratatui writes escape sequences into a Vec, which
// `Conn::render` drains and sends over the SSH channel after each draw.
type Term = Terminal<CrosstermBackend<Vec<u8>>>;

struct Conn {
    content: Arc<Content>,
    size: (u16, u16),
    term: Option<Term>,
    app: App,
}

// Posts shown per blog-index page; digits 1-9 then 0 select the ten slots.
const PAGE_SIZE: usize = 10;

// Which screen the session is showing. Indices point into content.pages /
// content.posts; the blog list's current page is App.blog_page.
#[derive(Clone, Copy)]
enum Screen {
    Page(usize),
    BlogIndex,
    Post(usize),
}

// UI state, kept separate from the SSH plumbing so its navigation logic is
// testable without a live session. `content_h`/`content_lines` are recorded on
// each render so key handling can page/clamp scrolling against the real layout.
struct App {
    screen: Screen,
    blog_page: usize,
    scroll: u16,
    content_h: u16,
    content_lines: u16,
}

impl App {
    const fn new() -> Self {
        Self {
            screen: Screen::Page(0),
            blog_page: 0,
            scroll: 0,
            content_h: 0,
            content_lines: 0,
        }
    }

    const fn open_page(&mut self, idx: usize) {
        self.screen = Screen::Page(idx);
        self.scroll = 0;
    }

    // Enter the blog index fresh (from nav), resetting to the first list page.
    const fn open_blog(&mut self) {
        self.screen = Screen::BlogIndex;
        self.blog_page = 0;
        self.scroll = 0;
    }

    // Return to the blog index from a post, keeping the list page we came from.
    const fn back_to_blog(&mut self) {
        self.screen = Screen::BlogIndex;
        self.scroll = 0;
    }

    const fn open_post(&mut self, idx: usize) {
        self.screen = Screen::Post(idx);
        self.scroll = 0;
    }

    const fn next_blog_page(&mut self, post_count: usize) {
        if self.blog_page.saturating_add(1).saturating_mul(PAGE_SIZE) < post_count {
            self.blog_page = self.blog_page.saturating_add(1);
        }
    }

    const fn prev_blog_page(&mut self) {
        self.blog_page = self.blog_page.saturating_sub(1);
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
                // CSI escape sequences: arrows (ESC [ A/B scroll a line) and
                // PageUp/Down (ESC [ 5~/6~ scroll a page). Scrolling applies on
                // every screen; a lone ESC (no `[`) falls through to handle_key.
                // Any other CSI is consumed whole up to its final byte, so its
                // tail can't leak back in as keystrokes (e.g. Right arrow's `C`
                // being read as a nav key).
                // ponytail: a sequence split across TCP reads is still dropped;
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
                        // Unrecognized: skip params/intermediates (0x20..=0x3f)
                        // to the final byte (0x40..=0x7e) and consume through it.
                        _ => {
                            i = csi_end(data, i);
                            continue;
                        }
                    }
                    i = i.saturating_add(3);
                    dirty = true;
                }
                // Everything else is screen-specific (nav keys, digits, paging).
                _ => {
                    if self.handle_key(b) {
                        dirty = true;
                    }
                    i = i.saturating_add(1);
                }
            }
        }
        if dirty {
            self.render(channel, session)?;
        }
        Ok(())
    }
}

// A single navigation target reachable from the Page screen's footer keys.
enum Target {
    Page(usize),
    Blog,
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
        let content = self.content.as_ref();
        term.draw(|f| ui(f, app, content))?;
        let buf = std::mem::take(term.backend_mut().writer_mut());
        if !buf.is_empty() {
            session.data(channel, buf)?;
        }
        Ok(())
    }

    // Handle one non-escape byte, dispatched by the current screen. Returns
    // whether it changed anything (so the caller knows to redraw). Space pages
    // the scrollable screens down; other keys are screen-specific.
    fn handle_key(&mut self, b: u8) -> bool {
        let step = self.app.content_h.max(1);
        let posts = self.content.posts.len();
        match self.app.screen {
            Screen::Page(_) => match b {
                b' ' => {
                    self.app.scroll_down(step);
                    true
                }
                // A nav key jumps to a page or the blog index.
                _ => match self.nav_key_target(char::from(b).to_ascii_lowercase()) {
                    Some(Target::Page(idx)) => {
                        self.app.open_page(idx);
                        true
                    }
                    Some(Target::Blog) => {
                        self.app.open_blog();
                        true
                    }
                    None => false,
                },
            },
            Screen::BlogIndex => match b {
                b'>' | b'.' => {
                    self.app.next_blog_page(posts);
                    true
                }
                b'<' | b',' => {
                    self.app.prev_blog_page();
                    true
                }
                // 'h' or Esc leaves the index for the home page.
                b'h' | 0x1b => {
                    self.app.open_page(0);
                    true
                }
                // A digit opens the matching entry on the current list page.
                _ => match digit_offset(b) {
                    Some(off) => {
                        let idx = self
                            .app
                            .blog_page
                            .saturating_mul(PAGE_SIZE)
                            .saturating_add(off);
                        if idx < posts {
                            self.app.open_post(idx);
                            true
                        } else {
                            false
                        }
                    }
                    None => false,
                },
            },
            Screen::Post(_) => match b {
                b' ' => {
                    self.app.scroll_down(step);
                    true
                }
                // 'b' or Esc returns to the blog index.
                b'b' | 0x1b => {
                    self.app.back_to_blog();
                    true
                }
                _ => false,
            },
        }
    }

    // Resolve a nav key to its target: one of the static pages, or the blog index.
    fn nav_key_target(&self, key: char) -> Option<Target> {
        let idx = nav_keys(&self.content.pages)
            .iter()
            .position(|k| *k == Some(key))?;
        if idx < self.content.pages.len() {
            Some(Target::Page(idx))
        } else {
            Some(Target::Blog)
        }
    }
}

// Given `start` at the ESC of a `ESC [ …` sequence, return the index just past
// the sequence's final byte. CSI params/intermediates are 0x20..=0x3f; the final
// byte is 0x40..=0x7e. If the chunk ends mid-sequence, returns the end (drop it).
fn csi_end(data: &[u8], start: usize) -> usize {
    let mut j = start.saturating_add(2); // skip ESC and '['
    while matches!(data.get(j), Some(0x20..=0x3f)) {
        j = j.saturating_add(1);
    }
    // j is at the final byte (or past the chunk); consume through it.
    j.saturating_add(1)
}

// Map an entry-select key to a 0-based slot on the current list page: '1'-'9'
// select 0-8, '0' selects the tenth. Anything else is not a selection key.
fn digit_offset(b: u8) -> Option<usize> {
    match b {
        b'1'..=b'9' => Some(usize::from(b.saturating_sub(b'1'))),
        b'0' => Some(9),
        _ => None,
    }
}

// Clamp a client-supplied PTY dimension into a sane terminal size. The upper
// bound matters for safety, not just sanity: ratatui eagerly allocates two
// width*height cell buffers, so an unclamped size (up to u16::MAX each) would
// let one connection request a multi-gigabyte allocation and OOM the process.
// MAX_DIM caps a single terminal's buffers at ~2*500*500 cells.
fn dim(v: u32) -> u16 {
    const MAX_DIM: u16 = 500;
    // Values above u16::MAX saturate, then clamp bounds them into [1, MAX_DIM].
    u16::try_from(v).unwrap_or(u16::MAX).clamp(1, MAX_DIM)
}

// Max reading-column width; the column is centered when the terminal is wider.
const CONTENT_WIDTH: u16 = 80;

fn ui(frame: &mut Frame, app: &mut App, content: &Content) {
    // Center a max-width column; body and footer both live inside it.
    let [column] = Layout::horizontal([Constraint::Max(CONTENT_WIDTH)])
        .flex(Flex::Center)
        .areas(frame.area());
    let [body, foot] = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(column);

    // Body text and footer both depend on which screen we're on.
    let (text, foot_text) = match app.screen {
        Screen::Page(idx) => (
            content
                .pages
                .get(idx)
                .map(|p| render_text(&p.body))
                .unwrap_or_default(),
            page_footer(&content.pages),
        ),
        Screen::BlogIndex => (
            blog_index_text(&content.posts, app.blog_page),
            blog_footer(&content.posts, app.blog_page),
        ),
        Screen::Post(idx) => (
            content.posts.get(idx).map(post_text).unwrap_or_default(),
            "[b] blog   [space] scroll   [q] quit".to_owned(),
        ),
    };

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    let lines = u16::try_from(paragraph.line_count(body.width)).unwrap_or(u16::MAX);

    // Record layout for the next key event and clamp scroll to the content.
    app.content_h = body.height;
    app.content_lines = lines;
    app.scroll = app.scroll.min(lines.saturating_sub(body.height));

    // Body left-aligned (centered prose reads badly); footer centered.
    frame.render_widget(paragraph.scroll((app.scroll, 0)), body);
    frame.render_widget(
        Paragraph::new(foot_text)
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM)),
        foot,
    );
}

// Assign each label a unique nav key: the first of its letters not already
// taken, scanning left to right. Collisions (Colophon and Contact both want
// 'c') fall through to the next free letter, so every entry stays reachable.
// 'q' is pre-reserved for quit. None if all a label's letters are taken.
fn assign_keys(labels: &[&str]) -> Vec<Option<char>> {
    let mut used = vec!['q'];
    labels
        .iter()
        .map(|label| {
            let key = label
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

// Nav keys for the Page screen: one per static page, plus a trailing "Blog"
// target. The returned vec is aligned with `pages` followed by Blog (last).
fn nav_keys(pages: &[Page]) -> Vec<Option<char>> {
    let mut labels: Vec<&str> = pages.iter().map(|p| p.nav.as_str()).collect();
    labels.push("Blog");
    assign_keys(&labels)
}

// Page-screen footer: each nav key + label (pages then Blog), then scroll/quit.
fn page_footer(pages: &[Page]) -> String {
    let mut labels: Vec<&str> = pages.iter().map(|p| p.nav.as_str()).collect();
    labels.push("Blog");
    let mut out = String::new();
    for (label, key) in labels.iter().zip(nav_keys(pages)) {
        if let Some(key) = key {
            out.push('[');
            out.push(key);
            out.push_str("] ");
            out.push_str(&label.to_lowercase());
            out.push_str("  ");
        }
    }
    out.push_str("[space] scroll  [q] quit");
    out
}

// The blog index body: a header and the current page's 10 numbered entries.
fn blog_index_text(posts: &[Post], blog_page: usize) -> String {
    use std::fmt::Write as _;
    let total_pages = posts.len().div_ceil(PAGE_SIZE).max(1);
    let start = blog_page.saturating_mul(PAGE_SIZE);
    let mut out = format!(
        "Blog — page {}/{}\n\n",
        blog_page.saturating_add(1),
        total_pages
    );
    for (i, post) in posts.iter().skip(start).take(PAGE_SIZE).enumerate() {
        // Slot labels are 1-9 then 0 for the tenth, matching digit_offset.
        let slot = if i == 9 { 0 } else { i.saturating_add(1) };
        // Writing to a String is infallible; discard the formatter Result.
        let _ = writeln!(out, "  {slot}. {}  ({})", post.title, post.date);
    }
    out
}

// Blog-index footer: entry-open hint, prev/next only when a page exists there.
fn blog_footer(posts: &[Post], blog_page: usize) -> String {
    let mut out = String::from("[1-0] open  ");
    if blog_page > 0 {
        out.push_str("[<] prev  ");
    }
    if blog_page.saturating_add(1).saturating_mul(PAGE_SIZE) < posts.len() {
        out.push_str("[>] next  ");
    }
    out.push_str("[h] home  [q] quit");
    out
}

// A post rendered for the terminal: title, date, then the body as plain text.
fn post_text(post: &Post) -> String {
    format!(
        "{}\n{}\n\n{}",
        post.title,
        post.date,
        render_text(&post.body)
    )
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
    use super::{
        App, CLEAR, Content, Page, Post, Screen, blog_index_text, csi_end, digit_offset, dim,
        nav_keys, page_footer, post_text, render_text, ui,
    };
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

    fn post(title: &str, date: &str) -> Post {
        Post {
            title: title.to_owned(),
            date: date.to_owned(),
            slug: title.to_ascii_lowercase(),
            body: format!("Body of {title}."),
        }
    }

    // N posts named "Post 1".."Post N", newest-date first (as load() delivers).
    fn posts(n: usize) -> Vec<Post> {
        (0..n)
            .map(|i| post(&format!("Post {i}"), "2025-01-01"))
            .collect()
    }

    fn content(pages: Vec<Page>, posts: Vec<Post>) -> Content {
        Content { pages, posts }
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
    fn page_footer_lists_keys_including_blog() {
        let pages = vec![page("Home", ""), page("About", "")];
        assert_eq!(
            page_footer(&pages),
            "[h] home  [a] about  [b] blog  [space] scroll  [q] quit"
        );
    }

    #[test]
    fn nav_keys_resolve_collisions_and_append_blog() {
        // Colophon takes 'c'; Contact falls through to 'o'; Blog gets 'b'.
        let pages = vec![page("Colophon", ""), page("Contact", "")];
        assert_eq!(
            nav_keys(&pages),
            vec![Some('c'), Some('o'), Some('b')],
            "third key is the appended Blog target"
        );
    }

    #[test]
    fn csi_end_consumes_whole_sequences() {
        // Right arrow ESC[C: final byte at index 2, consumed through it.
        assert_eq!(csi_end(b"\x1b[C", 0), 3);
        // Delete ESC[3~: one param then '~' final.
        assert_eq!(csi_end(b"\x1b[3~", 0), 4);
        // Modified ESC[5;2~: params '5' ';' '2' then '~'.
        assert_eq!(csi_end(b"\x1b[5;2~", 0), 6);
        // Truncated (no final byte in the chunk) consumes to the end.
        assert_eq!(csi_end(b"\x1b[3", 0), 4);
    }

    #[test]
    fn digit_offset_maps_slots() {
        assert_eq!(digit_offset(b'1'), Some(0));
        assert_eq!(digit_offset(b'9'), Some(8));
        assert_eq!(digit_offset(b'0'), Some(9), "'0' is the tenth slot");
        assert_eq!(digit_offset(b'a'), None);
    }

    #[test]
    fn screen_transitions_reset_scroll() {
        let mut app = App::new();
        app.scroll = 4;
        app.open_blog();
        assert!(matches!(app.screen, Screen::BlogIndex));
        assert_eq!(app.scroll, 0);
        assert_eq!(app.blog_page, 0);

        app.scroll = 2;
        app.open_post(3);
        assert!(matches!(app.screen, Screen::Post(3)));
        assert_eq!(app.scroll, 0);
    }

    #[test]
    fn blog_pagination_clamps_to_post_count() {
        let mut app = App::new();
        // 12 posts => two pages (0 and 1); can't advance past the last.
        app.next_blog_page(12);
        assert_eq!(app.blog_page, 1);
        app.next_blog_page(12);
        assert_eq!(app.blog_page, 1, "no page beyond the last");
        app.prev_blog_page();
        assert_eq!(app.blog_page, 0);
        app.prev_blog_page();
        assert_eq!(app.blog_page, 0, "no page before the first");
    }

    #[test]
    fn blog_index_numbers_current_page() {
        let ps = posts(12);
        let page0 = blog_index_text(&ps, 0);
        assert!(page0.contains("page 1/2"), "header shows position");
        assert!(page0.contains("1. Post 0"), "first slot is 1");
        assert!(page0.contains("0. Post 9"), "tenth slot is 0");
        assert!(!page0.contains("Post 10"), "page 1 stops at ten entries");

        let page1 = blog_index_text(&ps, 1);
        assert!(page1.contains("page 2/2"));
        assert!(
            page1.contains("1. Post 10"),
            "second page continues numbering at 1"
        );
    }

    #[test]
    fn scroll_clamps_to_content() {
        let mut app = App::new();
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
    fn post_text_shows_title_and_date() {
        let out = post_text(&post("Hello", "2025-06-01"));
        assert!(out.starts_with("Hello\n2025-06-01"), "title then date");
        assert!(out.contains("Body of Hello."));
    }

    #[test]
    fn ui_renders_every_screen_without_panicking() {
        let c = content(
            vec![
                page("Home", "# Welcome\n\nHello"),
                page("About", "About me"),
            ],
            posts(12),
        );
        // Each screen, at a normal and a degenerate size: layout math must not
        // panic (a clippy-denied subtraction underflow would surface here).
        for screen in [Screen::Page(0), Screen::BlogIndex, Screen::Post(0)] {
            for (w, h) in [(60, 20), (1, 1)] {
                let mut app = App::new();
                app.screen = screen;
                // TestBackend is infallible, so `match e {}` discharges the Result.
                let mut term = Terminal::new(TestBackend::new(w, h)).unwrap_or_else(|e| match e {});
                term.draw(|f| ui(f, &mut app, &c))
                    .unwrap_or_else(|e| match e {});
            }
        }
    }

    // Regression: the real CrosstermBackend render path (raw CLEAR + hide_cursor
    // + draw over a Fixed viewport) must succeed and emit content. Terminal::clear()
    // used to be here and errored with ENXIO on a headless server, killing the
    // session before anything rendered. Also checks the blog index renders posts.
    #[test]
    fn crossterm_backend_renders_content() -> Result<(), russh::Error> {
        use ratatui::backend::CrosstermBackend;
        use ratatui::layout::Rect;
        use ratatui::{TerminalOptions, Viewport};

        let c = content(vec![page("Home", "# Welcome\n\nHello there")], posts(12));
        let mut app = App::new();
        app.screen = Screen::BlogIndex;
        let mut term = Terminal::with_options(
            CrosstermBackend::new(Vec::new()),
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::new(0, 0, 80, 24)),
            },
        )?;
        term.hide_cursor()?;
        term.backend_mut().writer_mut().extend_from_slice(CLEAR);
        term.draw(|f| ui(f, &mut app, &c))?;

        // Note: ratatui's cell-diff skips space cells matching the empty
        // baseline, so a cursor move can split "Post 0" in the raw ANSI —
        // assert on single-token words, which stay contiguous.
        let ansi = String::from_utf8_lossy(term.backend_mut().writer_mut());
        assert!(ansi.contains("\x1b[2J"), "clears the client screen");
        assert!(ansi.contains("Blog"), "blog index header rendered");
        assert!(ansi.contains("Post"), "a post entry rendered");
        assert!(ansi.contains("open"), "blog footer hint rendered");
        Ok(())
    }
}
