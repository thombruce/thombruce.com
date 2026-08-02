//! SSH frontend — serves the site as a minimal TUI over SSH.
//!
//! Raw ANSI with single-key navigation: each page is reachable by a unique nav
//! key (the first free letter of its nav label; collisions fall through),
//! generated from the discovered pages (so adding a page needs no change here).
//! A ratatui menu is deferred (see issue #2).
//! Public access: any auth method is accepted.

use std::net::SocketAddr;
use std::sync::Arc;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use russh::keys::{PrivateKey, ssh_key};
use russh::server::ChannelOpenHandle;
use russh::server::{Auth, Config, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId};

use crate::content::Page;

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
        }
    }
}

struct Conn {
    pages: Arc<Vec<Page>>,
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

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_success(channel)?;
        if let Some(first) = self.pages.first() {
            session.data(channel, page_screen(first, &self.pages))?;
        }
        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        for &byte in data {
            // q, Q, Ctrl-C, Ctrl-D quit.
            if matches!(byte, b'q' | b'Q' | 3 | 4) {
                session.data(channel, b"\r\nBye.\r\n".to_vec())?;
                session.close(channel)?;
                continue;
            }
            let key = char::from(byte).to_ascii_lowercase();
            let hit = self
                .pages
                .iter()
                .zip(assign_keys(&self.pages))
                .find_map(|(p, k)| (k == Some(key)).then_some(p));
            if let Some(page) = hit {
                session.data(channel, page_screen(page, &self.pages))?;
            }
        }
        Ok(())
    }
}

// Assign each page a unique nav key: the first letter of its nav label that
// isn't already taken, scanning left to right. Collisions (Colophon and
// Contact both want 'c') fall through to the next free letter, so every page
// stays reachable. 'q' is pre-reserved for quit. None if all letters are taken.
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

// A footer listing every page's key + label, then quit — generated from pages.
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
    out.push_str("[q] quit");
    out
}

// Render a page's Markdown to terminal text and wrap it in a screen.
fn page_screen(page: &Page, pages: &[Page]) -> Vec<u8> {
    screen(&render_text(&page.body), &footer(pages))
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

// Clear screen, render body (PTYs need CRLF), append the nav footer.
fn screen(body: &str, footer: &str) -> Vec<u8> {
    let mut out = String::from("\x1b[2J\x1b[H");
    out.push_str(&body.replace('\n', "\r\n"));
    out.push_str("\r\n\r\n");
    out.push_str(footer);
    out.push_str("\r\n");
    out.into_bytes()
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::{Page, footer, render_text, screen};

    fn page(nav: &str) -> Page {
        Page {
            title: nav.to_owned(),
            nav: nav.to_owned(),
            order: 0,
            path: String::new(),
            body: String::new(),
        }
    }

    #[test]
    fn footer_lists_page_keys_then_quit() {
        let pages = vec![page("Home"), page("About")];
        assert_eq!(footer(&pages), "[h] home  [a] about  [q] quit");
    }

    #[test]
    fn footer_resolves_first_letter_collisions() {
        // Colophon takes 'c'; Contact falls through to its next free letter 'o'.
        let pages = vec![page("Colophon"), page("Contact")];
        assert_eq!(footer(&pages), "[c] colophon  [o] contact  [q] quit");
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
    fn screen_clears_converts_newlines_and_appends_footer() -> Result<(), std::string::FromUtf8Error>
    {
        let out = String::from_utf8(screen("a\nb", "[q] quit"))?;

        assert!(
            out.starts_with("\x1b[2J\x1b[H"),
            "should clear the screen first"
        );
        assert!(out.contains("a\r\nb"), "newlines become CRLF for the PTY");
        assert!(out.ends_with("[q] quit\r\n"), "footer appended");
        Ok(())
    }
}
