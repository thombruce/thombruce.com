---
title: Tonic
repo: thombruce/tonic
language: Rust
---
# Tonic

A git worktree companion: `git` and `tonic`.

Tonic is about moving around your git work locally, along two axes:

- **Lateral:** jump between worktrees with `add`, `list`, `cd`, and `rm`. Shell integration drops you into the right directory.
- **Vertical:** navigate stacked branches with `up`, `down`, `top`, and `bottom`, inferred from the commit graph. It never rebases for you.

It also respects `.worktreeinclude` (copying or symlinking your gitignored files into new worktrees) and runs lifecycle hooks on create and remove, so per-worktree databases or containers spin up and tear down cleanly.

```sh
brew install thombruce/tap/tonic
# or
cargo install git-tonic

# then, in ~/.zshrc or ~/.bashrc
eval "$(tonic shell-init zsh)"
```

Source: [thombruce/tonic](https://github.com/thombruce/tonic)
