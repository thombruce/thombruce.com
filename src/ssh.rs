//! SSH frontend — serves the site as a ratatui TUI over SSH.
//!
//! Two screens, each a scrollable text view with a key-hint footer:
//! - **Doc** — a single document. At the content root the footer shows the nav
//!   (each entry's key is the first free letter of its label; collisions fall
//!   through). Nested docs show `b` back to their listing, `h` home.
//! - **Listing** — a directory's paginated entries (10/page); digits `1`-`9`/`0`
//!   open an entry (doc or subdirectory), `>`/`<` page, `b` up, `h` home.
//!
//! Nav and lists derive from the discovered content, so adding content needs
//! no change here. Rendered by ratatui through a `CrosstermBackend`
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
use ratatui::text::{Line, Text};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{Frame, TerminalOptions, Viewport};
use russh::keys::{PrivateKey, ssh_key};
use russh::server::ChannelOpenHandle;
use russh::server::{Auth, Config, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId, Pty};

use crate::content::{Content, Doc, Entry, Listing, NavLink, Target, strip_leading_h1};

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
            app: App::new(self.content.home.unwrap_or(0)),
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

// Entries shown per listing page; digits 1-9 then 0 select the ten slots.
const PAGE_SIZE: usize = 10;

// Which screen the session is showing, as an index into content.docs /
// content.listings; a listing's current page is App.list_page, which belongs
// to listing App.list_of.
#[derive(Clone, Copy)]
enum Screen {
    Doc(usize),
    Listing(usize),
}

// UI state, kept separate from the SSH plumbing so its navigation logic is
// testable without a live session. `content_h`/`content_lines` are recorded on
// each render so key handling can page/clamp scrolling against the real layout.
struct App {
    screen: Screen,
    home: usize,
    list_page: usize,
    list_of: usize,
    scroll: u16,
    content_h: u16,
    content_lines: u16,
}

impl App {
    const fn new(home: usize) -> Self {
        Self {
            screen: Screen::Doc(home),
            home,
            list_page: 0,
            list_of: 0,
            scroll: 0,
            content_h: 0,
            content_lines: 0,
        }
    }

    // Open a doc, or enter a listing fresh at its first page.
    const fn open(&mut self, target: Target) {
        self.screen = match target {
            Target::Doc(idx) => Screen::Doc(idx),
            Target::Listing(idx) => {
                self.list_page = 0;
                self.list_of = idx;
                Screen::Listing(idx)
            }
        };
        self.scroll = 0;
    }

    // Return from a doc to its listing, keeping the list page we came from —
    // unless that page belonged to a different listing (the doc was reached
    // some other way, e.g. by nav key), in which case start at its first page.
    const fn back_to_listing(&mut self, idx: usize) {
        if self.list_of != idx {
            self.list_page = 0;
            self.list_of = idx;
        }
        self.screen = Screen::Listing(idx);
        self.scroll = 0;
    }

    const fn go_home(&mut self) {
        self.open(Target::Doc(self.home));
    }

    const fn next_list_page(&mut self, entry_count: usize) {
        if self.list_page.saturating_add(1).saturating_mul(PAGE_SIZE) < entry_count {
            self.list_page = self.list_page.saturating_add(1);
        }
    }

    const fn prev_list_page(&mut self) {
        self.list_page = self.list_page.saturating_sub(1);
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
    // docs down; other keys are screen-specific.
    fn handle_key(&mut self, b: u8) -> bool {
        let step = self.app.content_h.max(1);
        match self.app.screen {
            Screen::Doc(idx) => {
                let parent = self.content.docs.get(idx).and_then(|d| d.parent);
                match (b, parent) {
                    (b' ', _) => self.app.scroll_down(step),
                    // 'b' or Esc returns to the doc's listing; 'h' goes home.
                    (b'b' | 0x1b, Some(p)) => self.app.back_to_listing(p),
                    (b'h', Some(_)) => self.app.go_home(),
                    // At the root, a nav key jumps to its doc or listing.
                    (_, None) => match self.nav_key_target(char::from(b).to_ascii_lowercase()) {
                        Some(target) => self.app.open(target),
                        None => return false,
                    },
                    _ => return false,
                }
            }
            Screen::Listing(idx) => {
                let Some(listing) = self.content.listings.get(idx) else {
                    return false;
                };
                match b {
                    b'>' | b'.' => self.app.next_list_page(listing.entries.len()),
                    b'<' | b',' => self.app.prev_list_page(),
                    // 'b' or Esc goes up a level (home from a top-level listing).
                    b'b' | 0x1b => match listing.parent {
                        Some(p) => self.app.open(Target::Listing(p)),
                        None => self.app.go_home(),
                    },
                    b'h' => self.app.go_home(),
                    // A digit opens the matching entry on the current list page.
                    _ => {
                        let entry = digit_offset(b).and_then(|off| {
                            listing.entries.get(
                                self.app
                                    .list_page
                                    .saturating_mul(PAGE_SIZE)
                                    .saturating_add(off),
                            )
                        });
                        match entry {
                            Some(e) => self.app.open(e.target),
                            None => return false,
                        }
                    }
                }
            }
        }
        true
    }

    // Resolve a nav key to the doc or listing it opens.
    fn nav_key_target(&self, key: char) -> Option<Target> {
        let idx = nav_keys(&self.content.nav)
            .iter()
            .position(|k| *k == Some(key))?;
        self.content.nav.get(idx).map(|n| n.target)
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

// Max footer height; past this the footer clips rather than squeeze the body.
const MAX_FOOTER_LINES: u16 = 3;

fn ui(frame: &mut Frame, app: &mut App, content: &Content) {
    // Body text and footer both depend on which screen we're on.
    let (text, foot_text) = match app.screen {
        Screen::Doc(idx) => content.docs.get(idx).map_or_else(
            || (Text::default(), nav_footer(&content.nav)),
            |doc| (doc_text(doc), doc_footer(doc, content)),
        ),
        Screen::Listing(idx) => content
            .listings
            .get(idx)
            .map(|l| {
                (
                    Text::from(listing_text(l, app.list_page, content)),
                    listing_footer(l, app.list_page),
                )
            })
            .unwrap_or_default(),
    };

    // Footer spans the full terminal width (not the reading column) and wraps
    // when even that is too narrow. NBSP glues each "[k]" to its label, so a
    // wrap never splits a hint from its key.
    let footer = Paragraph::new(foot_text.replace("] ", "]\u{a0}"))
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::DIM))
        .wrap(Wrap { trim: true });
    let foot_h = u16::try_from(footer.line_count(frame.area().width))
        .unwrap_or(u16::MAX)
        .min(MAX_FOOTER_LINES);
    let [main, foot] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(foot_h)]).areas(frame.area());
    // Only the body is held to a centered max-width reading column.
    let [body] = Layout::horizontal([Constraint::Max(CONTENT_WIDTH)])
        .flex(Flex::Center)
        .areas(main);

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    let lines = u16::try_from(paragraph.line_count(body.width)).unwrap_or(u16::MAX);

    // Record layout for the next key event and clamp scroll to the content.
    app.content_h = body.height;
    app.content_lines = lines;
    app.scroll = app.scroll.min(lines.saturating_sub(body.height));

    // Body left-aligned (centered prose reads badly); footer centered.
    frame.render_widget(paragraph.scroll((app.scroll, 0)), body);
    frame.render_widget(footer, foot);
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

// Nav keys, aligned with the content's nav entries.
fn nav_keys(nav: &[NavLink]) -> Vec<Option<char>> {
    let labels: Vec<&str> = nav.iter().map(|n| n.label.as_str()).collect();
    assign_keys(&labels)
}

// Root-doc footer: each nav key + label, then scroll/quit.
fn nav_footer(nav: &[NavLink]) -> String {
    let mut out = String::new();
    for (link, key) in nav.iter().zip(nav_keys(nav)) {
        if let Some(key) = key {
            out.push('[');
            out.push(key);
            out.push_str("] ");
            out.push_str(&link.label.to_lowercase());
            out.push_str("  ");
        }
    }
    out.push_str("[space] scroll  [q] quit");
    out
}

// A doc's footer: the nav at the root; otherwise back to its listing, and home.
fn doc_footer(doc: &Doc, content: &Content) -> String {
    doc.parent
        .and_then(|p| content.listings.get(p))
        .map_or_else(
            || nav_footer(&content.nav),
            |listing| {
                format!(
                    "[b] {}  [h] home  [space] scroll  [q] quit",
                    listing.title.to_lowercase()
                )
            },
        )
}

// A listing's body: its intro (or title), then the current page's numbered
// entries, with a page counter when there's more than one page.
// A listing's text version can't replace the screen (its numbered slots are
// what the digit keys select), so a listing template adds detail lines under
// each entry instead. Same names as the HTML index templates (view.rs); a
// listing without one shows bare entries.
type EntryDetail = fn(&Entry, &Content) -> Vec<String>;
const LISTING_TEXT_TEMPLATES: [(&str, EntryDetail); 1] = [("projects", project_detail)];

// `projects`: the entry's language and repo (for docs), then its summary.
fn project_detail(entry: &Entry, content: &Content) -> Vec<String> {
    let details = content.doc_meta(entry.target).map(|meta| {
        [meta.get("language"), meta.get("repo")]
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" · ")
    });
    details
        .filter(|d| !d.is_empty())
        .into_iter()
        .chain(content.summary(entry.target))
        .collect()
}

fn listing_text(listing: &Listing, list_page: usize, content: &Content) -> String {
    use std::fmt::Write as _;
    let detail = LISTING_TEXT_TEMPLATES
        .iter()
        .find(|(name, _)| Some(*name) == listing.layout.as_deref())
        .map(|(_, f)| *f);
    let mut out = if listing.intro.trim().is_empty() {
        listing.title.clone()
    } else {
        render_text(&listing.intro)
    };
    out.push_str("\n\n");
    if listing.entries.is_empty() {
        out.push_str("Nothing here yet.");
        return out;
    }
    let total_pages = listing.entries.len().div_ceil(PAGE_SIZE);
    // Writing to a String is infallible; discard the formatter Results.
    if total_pages > 1 {
        let _ = writeln!(out, "Page {}/{total_pages}\n", list_page.saturating_add(1));
    }
    let start = list_page.saturating_mul(PAGE_SIZE);
    for (i, entry) in listing
        .entries
        .iter()
        .skip(start)
        .take(PAGE_SIZE)
        .enumerate()
    {
        // Slot labels are 1-9 then 0 for the tenth, matching digit_offset.
        let slot = if i == 9 { 0 } else { i.saturating_add(1) };
        let _ = write!(out, "  {slot}. {}", entry.title);
        if let Some(date) = &entry.date {
            let _ = write!(out, "  ({date})");
        }
        out.push('\n');
        for line in detail.map(|f| f(entry, content)).unwrap_or_default() {
            let _ = writeln!(out, "     {line}");
        }
    }
    out
}

// Listing footer: open/paging hints only where they apply.
fn listing_footer(listing: &Listing, list_page: usize) -> String {
    let mut out = String::new();
    if !listing.entries.is_empty() {
        out.push_str("[1-0] open  ");
    }
    if list_page > 0 {
        out.push_str("[<] prev  ");
    }
    if list_page.saturating_add(1).saturating_mul(PAGE_SIZE) < listing.entries.len() {
        out.push_str("[>] next  ");
    }
    if listing.parent.is_some() {
        out.push_str("[b] back  ");
    }
    out.push_str("[h] home  [q] quit");
    out
}

// A doc rendered for the terminal: by the text version of its template if
// there is one, else generically. Text versions share the HTML templates'
// names (see view.rs), so `layout: post` selects both; one without a text
// version falls back here rather than breaking SSH.
// (Listings work differently: see LISTING_TEXT_TEMPLATES.)
fn doc_text(doc: &Doc) -> Text<'static> {
    TEXT_TEMPLATES
        .iter()
        .find(|(name, _)| Some(*name) == doc.layout.as_deref())
        .map_or_else(|| default_doc_text(doc), |(_, template)| template(doc))
}

type TextTemplate = fn(&Doc) -> Text<'static>;
const TEXT_TEMPLATES: [(&str, TextTemplate); 2] = [("post", post_text), ("project", project_text)];

// Generic: the date (if any), then the body as text.
fn default_doc_text(doc: &Doc) -> Text<'static> {
    let body = render_text(&doc.body);
    Text::from(match doc.date() {
        Some(date) => format!("{date}\n\n{body}"),
        None => body,
    })
}

// `post`: bold title, dimmed date, then the body (leading H1 dropped, as on
// the web, so the title isn't repeated).
fn post_text(doc: &Doc) -> Text<'static> {
    let mut lines = vec![Line::styled(
        doc.title.clone(),
        Style::default().add_modifier(Modifier::BOLD),
    )];
    if let Some(date) = doc.date() {
        lines.push(Line::styled(
            date.to_owned(),
            Style::default().add_modifier(Modifier::DIM),
        ));
    }
    lines.push(Line::default());
    lines.extend(Text::from(render_text(&strip_leading_h1(&doc.body))).lines);
    Text::from(lines)
}

// `project`: bold title, dimmed "language · repo", then the body (leading
// H1 dropped).
fn project_text(doc: &Doc) -> Text<'static> {
    let mut lines = vec![Line::styled(
        doc.title.clone(),
        Style::default().add_modifier(Modifier::BOLD),
    )];
    let details: Vec<&str> = [doc.meta.get("language"), doc.meta.get("repo")]
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect();
    if !details.is_empty() {
        lines.push(Line::styled(
            details.join(" · "),
            Style::default().add_modifier(Modifier::DIM),
        ));
    }
    lines.push(Line::default());
    lines.extend(Text::from(render_text(&strip_leading_h1(&doc.body))).lines);
    Text::from(lines)
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
        App, CLEAR, Content, Doc, Listing, NavLink, Screen, Target, csi_end, digit_offset, dim,
        doc_text, listing_footer, listing_text, nav_footer, nav_keys, render_text, ui,
    };
    use crate::content::Entry;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn doc(title: &str, body: &str, date: Option<&str>, parent: Option<usize>) -> Doc {
        Doc {
            path: format!("/{}", title.to_ascii_lowercase()),
            title: title.to_owned(),
            meta: date
                .map(|d| ("date".to_owned(), d.to_owned()))
                .into_iter()
                .collect(),
            body: body.to_owned(),
            parent,
            layout: None,
        }
    }

    fn nav(labels: &[&str]) -> Vec<NavLink> {
        labels
            .iter()
            .enumerate()
            .map(|(i, l)| NavLink {
                label: (*l).to_owned(),
                path: format!("/{}", l.to_ascii_lowercase()),
                target: Target::Doc(i),
            })
            .collect()
    }

    // A listing of `n` entries titled "Post 0".."Post n-1", each targeting doc i.
    fn listing(n: usize, parent: Option<usize>) -> Listing {
        Listing {
            path: "/blog".to_owned(),
            title: "Blog".to_owned(),
            intro: String::new(),
            entries: (0..n)
                .map(|i| Entry {
                    title: format!("Post {i}"),
                    path: format!("/blog/post-{i}"),
                    date: Some("2025-01-01".to_owned()),
                    target: Target::Doc(i),
                })
                .collect(),
            parent,
            layout: None,
        }
    }

    // Home + About at the root, then a 12-post Blog listing (listing 0) whose
    // entries are docs 2..14; nav: Home, About, Blog.
    fn content() -> Content {
        let mut docs = vec![
            doc("Home", "# Welcome\n\nHello", None, None),
            doc("About", "About me", None, None),
        ];
        let mut blog = listing(12, None);
        for (i, entry) in blog.entries.iter_mut().enumerate() {
            entry.target = Target::Doc(docs.len());
            docs.push(doc(
                &format!("Post {i}"),
                "Body.",
                Some("2025-01-01"),
                Some(0),
            ));
        }
        let mut links = nav(&["Home", "About", "Blog"]);
        if let Some(b) = links.get_mut(2) {
            b.target = Target::Listing(0);
        }
        Content {
            docs,
            listings: vec![blog],
            nav: links,
            home: Some(0),
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
    fn nav_footer_lists_keys() {
        assert_eq!(
            nav_footer(&nav(&["Home", "About", "Blog"])),
            "[h] home  [a] about  [b] blog  [space] scroll  [q] quit"
        );
    }

    #[test]
    fn nav_keys_resolve_collisions() {
        // Colophon takes 'c'; Contact falls through to 'o'; Code to 'd'.
        assert_eq!(
            nav_keys(&nav(&["Colophon", "Contact", "Code"])),
            vec![Some('c'), Some('o'), Some('d')]
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
    fn screen_transitions_reset_scroll_and_keep_list_page() {
        let mut app = App::new(0);
        app.scroll = 4;
        app.list_page = 3;
        app.open(Target::Listing(1));
        assert!(matches!(app.screen, Screen::Listing(1)));
        assert_eq!((app.scroll, app.list_page), (0, 0), "listing entered fresh");

        app.list_page = 1;
        app.scroll = 2;
        app.open(Target::Doc(3));
        assert!(matches!(app.screen, Screen::Doc(3)));
        assert_eq!(app.scroll, 0);
        app.back_to_listing(1);
        assert_eq!(app.list_page, 1, "back keeps the page we came from");

        // Back into a different listing than the page belonged to: first page.
        app.back_to_listing(2);
        assert!(matches!(app.screen, Screen::Listing(2)));
        assert_eq!(app.list_page, 0, "stale page from another listing reset");

        app.go_home();
        assert!(matches!(app.screen, Screen::Doc(0)));
    }

    #[test]
    fn list_pagination_clamps_to_entry_count() {
        let mut app = App::new(0);
        // 12 entries => two pages (0 and 1); can't advance past the last.
        app.next_list_page(12);
        assert_eq!(app.list_page, 1);
        app.next_list_page(12);
        assert_eq!(app.list_page, 1, "no page beyond the last");
        app.prev_list_page();
        assert_eq!(app.list_page, 0);
        app.prev_list_page();
        assert_eq!(app.list_page, 0, "no page before the first");
    }

    #[test]
    fn listing_empty_state() {
        let empty = listing(0, None);
        let body = listing_text(&empty, 0, &content());
        assert!(body.contains("Nothing here yet"), "empty body message");
        assert!(!body.contains("Page 1"), "no page counter when empty");
        assert_eq!(
            listing_footer(&empty, 0),
            "[h] home  [q] quit",
            "no open/paging/back hints when empty at the top level"
        );
        assert!(listing_footer(&listing(0, Some(0)), 0).contains("[b] back"));
    }

    #[test]
    fn listing_numbers_current_page() {
        let l = listing(12, None);
        let page0 = listing_text(&l, 0, &content());
        assert!(page0.starts_with("Blog"), "titled when there's no intro");
        assert!(page0.contains("Page 1/2"), "header shows position");
        assert!(
            page0.contains("1. Post 0  (2025-01-01)"),
            "first slot is 1, dated"
        );
        assert!(page0.contains("0. Post 9"), "tenth slot is 0");
        assert!(!page0.contains("Post 10"), "page 1 stops at ten entries");

        let page1 = listing_text(&l, 1, &content());
        assert!(page1.contains("Page 2/2"));
        assert!(
            page1.contains("1. Post 10"),
            "second page continues numbering at 1"
        );
    }

    #[test]
    fn scroll_clamps_to_content() {
        let mut app = App::new(0);
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

    // Text content, one string per line (styles dropped).
    fn plain(text: &ratatui::text::Text) -> Vec<String> {
        text.lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect()
    }

    #[test]
    fn doc_text_shows_date_when_present() {
        let dated = doc_text(&doc("Hello", "Body.", Some("2025-06-01"), None));
        assert_eq!(plain(&dated), ["2025-06-01", "", "Body."]);
        assert_eq!(
            plain(&doc_text(&doc("Hello", "Body.", None, None))),
            ["Body."]
        );
    }

    #[test]
    fn post_text_template_styles_title_without_repeating_it() {
        let mut post = doc("Hello", "# Hello\n\nBody.", Some("2025-06-01"), None);
        post.layout = Some("post".to_owned());
        let text = doc_text(&post);
        assert_eq!(plain(&text), ["Hello", "2025-06-01", "", "Body."]);
        let title_style = text.lines.first().map(|l| l.style);
        assert!(
            title_style.is_some_and(|s| s.add_modifier.contains(ratatui::style::Modifier::BOLD)),
            "title is bold"
        );
    }

    #[test]
    fn text_templates_name_real_layouts() {
        let layouts = crate::view::doc_layouts();
        for (name, _) in super::TEXT_TEMPLATES {
            assert!(
                layouts.contains(&name),
                "SSH template {name} has no HTML template"
            );
        }
        let index_layouts = crate::view::index_layouts();
        for (name, _) in super::LISTING_TEXT_TEMPLATES {
            assert!(
                index_layouts.contains(&name),
                "SSH listing template {name} has no HTML template"
            );
        }
    }

    #[test]
    fn project_templates_show_language_repo_and_summary() {
        let mut c = content();
        let mut project = doc("Tonic", "# Tonic\n\nA worktree companion.", None, None);
        project.layout = Some("project".to_owned());
        project
            .meta
            .insert("language".to_owned(), "Rust".to_owned());
        project
            .meta
            .insert("repo".to_owned(), "thombruce/tonic".to_owned());
        assert_eq!(
            plain(&doc_text(&project)),
            [
                "Tonic",
                "Rust · thombruce/tonic",
                "",
                "A worktree companion."
            ]
        );

        // A one-entry `projects` listing pointing at that doc.
        c.docs.push(project);
        let mut l = listing(0, None);
        l.layout = Some("projects".to_owned());
        l.entries.push(Entry {
            title: "Tonic".to_owned(),
            path: "/code/tonic".to_owned(),
            date: None,
            target: Target::Doc(c.docs.len().saturating_sub(1)),
        });
        let text = listing_text(&l, 0, &c);
        assert!(
            text.contains("1. Tonic\n     Rust · thombruce/tonic\n     A worktree companion."),
            "{text}"
        );
    }

    #[test]
    fn ui_renders_every_screen_without_panicking() {
        let c = content();
        // Each screen, at a normal and a degenerate size: layout math must not
        // panic (a clippy-denied subtraction underflow would surface here).
        for screen in [Screen::Doc(0), Screen::Listing(0), Screen::Doc(2)] {
            for (w, h) in [(60, 20), (1, 1)] {
                let mut app = App::new(0);
                app.screen = screen;
                // TestBackend is infallible, so `match e {}` discharges the Result.
                let mut term = Terminal::new(TestBackend::new(w, h)).unwrap_or_else(|e| match e {});
                term.draw(|f| ui(f, &mut app, &c))
                    .unwrap_or_else(|e| match e {});
            }
        }
    }

    // A footer wider than the terminal wraps instead of clipping: the last
    // hint must still be on screen, with no "[k]" split from its label.
    #[test]
    fn footer_wraps_on_narrow_terminal() {
        let c = Content {
            docs: vec![doc("Home", "", None, None)],
            listings: vec![],
            nav: nav(&["Home", "About", "Projects", "Contact"]),
            home: Some(0),
        };
        let mut app = App::new(0);
        let mut term = Terminal::new(TestBackend::new(40, 10)).unwrap_or_else(|e| match e {});
        term.draw(|f| ui(f, &mut app, &c))
            .unwrap_or_else(|e| match e {});
        let screen = term.backend().to_string();
        assert!(
            screen.contains("quit"),
            "last hint visible after wrapping:\n{screen}"
        );
        assert!(
            screen.contains("[p]\u{a0}projects"),
            "key stays glued to its label"
        );
    }

    // Regression: the real CrosstermBackend render path (raw CLEAR + hide_cursor
    // + draw over a Fixed viewport) must succeed and emit content. Terminal::clear()
    // used to be here and errored with ENXIO on a headless server, killing the
    // session before anything rendered. Also checks a listing renders entries.
    #[test]
    fn crossterm_backend_renders_content() -> Result<(), russh::Error> {
        use ratatui::backend::CrosstermBackend;
        use ratatui::layout::Rect;
        use ratatui::{TerminalOptions, Viewport};

        let c = content();
        let mut app = App::new(0);
        app.screen = Screen::Listing(0);
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
        assert!(ansi.contains("Blog"), "listing header rendered");
        assert!(ansi.contains("Post"), "an entry rendered");
        assert!(ansi.contains("open"), "listing footer hint rendered");
        Ok(())
    }
}
