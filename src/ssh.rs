//! SSH frontend — serves the site as a minimal TUI over SSH.
//!
//! v1 is raw ANSI + single-key navigation (h/a/q); a ratatui layout is
//! deferred (see issue #2). Public access: any auth method is accepted.

use std::net::SocketAddr;
use std::sync::Arc;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use russh::keys::{PrivateKey, ssh_key};
use russh::server::ChannelOpenHandle;
use russh::server::{Auth, Config, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId};

use crate::content::{self, Page};

pub async fn serve(addr: String) -> std::io::Result<()> {
    // Ephemeral host key: regenerated each boot. Fine for a public read-only
    // service; clients will just see a changed fingerprint after a redeploy.
    // ponytail: persist a key (env/secret) if stable fingerprints ever matter.
    let key = PrivateKey::random(&mut rand::rng(), ssh_key::Algorithm::Ed25519)
        .map_err(std::io::Error::other)?;
    let config = Arc::new(Config {
        keys: vec![key],
        ..Config::default()
    });

    let mut server = AppServer;
    server.run_on_address(config, addr).await
}

struct AppServer;

impl Server for AppServer {
    type Handler = Conn;

    fn new_client(&mut self, _peer: Option<SocketAddr>) -> Conn {
        Conn
    }
}

struct Conn;

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
        session.data(channel, page(&content::HOME))?;
        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        for &byte in data {
            match byte {
                b'h' | b'H' => session.data(channel, page(&content::HOME))?,
                b'a' | b'A' => session.data(channel, page(&content::ABOUT))?,
                // q, Q, Ctrl-C, Ctrl-D
                b'q' | b'Q' | 3 | 4 => {
                    session.data(channel, b"\r\nBye.\r\n".to_vec())?;
                    session.close(channel)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

// Render a page's Markdown to plain terminal text and wrap it in a screen.
fn page(page: &Page) -> Vec<u8> {
    screen(&render_text(page.markdown))
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

// Clear screen, render body (PTYs need CRLF), append a nav footer.
fn screen(body: &str) -> Vec<u8> {
    let mut out = String::from("\x1b[2J\x1b[H");
    out.push_str(&body.replace('\n', "\r\n"));
    out.push_str("\r\n\r\n[h] home  [a] about  [q] quit\r\n");
    out.into_bytes()
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::{render_text, screen};

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
        let out = String::from_utf8(screen("a\nb"))?;

        assert!(
            out.starts_with("\x1b[2J\x1b[H"),
            "should clear the screen first"
        );
        assert!(out.contains("a\r\nb"), "newlines become CRLF for the PTY");
        assert!(
            out.ends_with("[h] home  [a] about  [q] quit\r\n"),
            "footer appended"
        );
        Ok(())
    }
}
