use maud::{DOCTYPE, Markup, html};

// Shared page shell (doctype, head, nav chrome). Body passed by reference so
// callers can hand in an `html!` temporary without a needless move.
fn layout(title: &str, body: &Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " · Thom Bruce" }
            }
            body {
                nav {
                    a href="/" { "Home" }
                    " · "
                    a href="/about" { "About" }
                }
                main {
                    (body)
                }
            }
        }
    }
}

pub async fn home() -> Markup {
    layout(
        "Home",
        &html! {
            h1 { "Thom Bruce" }
            p { "Web and Software Developer" }
            p { "Ruby / Rust / TypeScript" }
            p { em { "Site Under Construction" } }
        },
    )
}

pub async fn about() -> Markup {
    layout(
        "About",
        &html! {
            h1 { "About" }
            ol type="i" {
                li { "Rubyist" }
                li { "Rustacean" }
                li { "TypeScripter" }
            }
            p { "Writing code for nearly 20 years." }
        },
    )
}
