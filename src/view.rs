use maud::{DOCTYPE, Markup, html};

// Shared HTML shell: doctype, head (stylesheet), nav chrome, main content.
// Body passed by reference so callers can hand in an `html!` temporary.
pub fn shell(title: &str, body: &Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                link rel="stylesheet" href="/style.css";
                title { (title) " · Thom Bruce" }
            }
            body {
                nav {
                    a href="/" { "Home" }
                    " · "
                    a href="/about" { "About" }
                    " · "
                    a href="/colophon" { "Colophon" }
                }
                main {
                    (body)
                }
            }
        }
    }
}
