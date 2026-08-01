//! Site content as data, decoupled from any frontend. Each page's body is
//! Markdown embedded at compile time; the HTTP frontend renders it to HTML and
//! the SSH frontend renders it to text, so the source lives in exactly one place.

pub struct Page {
    pub title: &'static str,
    pub markdown: &'static str,
}

pub const HOME: Page = Page {
    title: "Home",
    markdown: include_str!("../content/home.md"),
};

pub const ABOUT: Page = Page {
    title: "About",
    markdown: include_str!("../content/about.md"),
};
