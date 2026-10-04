---
title: Inkpot
repo: thombruce/inkpot
language: Rust
---
# Inkpot

A writing format and desktop app for prose and poetry. It keeps a whole work (a novel, a short story, an article) in one continuous plain-text document, while letting you rearrange its parts, annotate invisibly, and edit non-destructively.

The `.ink` format has **visible headings** that go to print (chapters, sections) and **invisible headings** that don't (scenes, beats), sharing one hierarchy:

```
# Chapter 1

~~ The Kitchen
time: dawn
pov: Alice

She stood at the counter. {+Steam rose from the kettle.}
{/is this too early in the timeline?}
```

From one source it generates:

- a clean **manuscript** (Markdown, or a Shunn-format PDF ready for submission)
- a structural **outline**
- a **codex** of characters, locations, and notes
- a **bibliography** of cited sources

The desktop app adds a timeline, character cards, and a map that follows your cast through the story.

Plain text, git-friendly, no lock-in.
