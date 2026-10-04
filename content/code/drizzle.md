---
title: Drizzle
repo: thombruce/drizzle
language: CSS
---
# Drizzle

A classless-first CSS framework driven by CSS variables. Bare semantic HTML renders well with no classes or IDs at all. This site is styled with it: there isn't a single `class` attribute in the markup.

- **Classless by default.** Headings, prose, forms, tables, lists, `details`, and `dialog` are all styled as written.
- **Variables everywhere.** Every color, size, weight, gutter, border, and radius is a CSS custom property, so retheming is a matter of overriding one:

  ```css
  :root {
    --color-link: #ff6b6b;
    --radius: 0;
  }
  ```

- **Dark mode.** Automatic via `prefers-color-scheme`, with `.dark` and `.light` overrides for a manual toggle.
- **Tiny.** A single file, no JavaScript, no dependencies for consumers.

Drop the stylesheet into any page, or use it from Rust: the `drizzle-css` crate bundles it at build time with [Lightning CSS](https://lightningcss.dev/) and embeds it as a `&'static str`. That's how this site serves its `/style.css`.

```toml
[dependencies]
drizzle-css = "0.1"
```
