use crate::content;

pub async fn home() -> &'static str {
    content::HOME
}

pub async fn about() -> &'static str {
    content::ABOUT
}
