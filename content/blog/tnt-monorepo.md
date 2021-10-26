---
title: "Creating a Monorepo for my TNT Packages"
description: 'Combining the files and histories of three separate Git repositories'
authors:
  - Thom Bruce
date: 2021-09-04T02:20:17Z
categories:
  - Journal
series:
  - My Process
tags:
  - Git
  - Vue.js
  - Vue CLI
  - NuxtJS
  - TNT
---

Recently, I [reworked TNT](/series/reworking-tnt). TNT was a Nuxt module that provided a set of common dependencies and components that I use frequently across my Nuxt projects. Now, it's a Vue plugin that does the same for Vue projects that do not use Nuxt. And it's also a Vue CLI plugin, which just makes installing it a lot easier. In fact, these three things - the Nuxt module, the Vue plugin and the Vue CLI plugin - now live in three separate repositories... but they share a lot of DNA, naturally. Both the Nuxt module and the CLI plugin require TNT as a dependency. And the TNT docs, which live in the main repository, in turn require Nuxt TNT... It's _almost_ a circular dependency. _Almost._ Strictly the docs site is a distinct project from the main TNT one, they just live alongside one another. They didn't have to live together - that's a decision I made so that the TNT docs would live at a nice looking GitHub Pages URL... It was a vanity. But it has other benefits, we're just not yet making full utilisation of them because of the aforementioned _almost_ circular dependency. The docs don't directly depend on the main code, they depend on Nuxt TNT which depends on the main code in turn. So... what if I moved Nuxt TNT (and the CLI plugin) into this same repository? Then I wouldn't have to worry about this project depending on that one, which in turn depends on the first one... The _almost_ circular dependency would instead refer to projects in the same project tree. The _almost_ circle is still there, but it's neatly hidden in a branching directory structure. My code could make use of the latest changes to other projects in the directory tree, common processes could be run from the parent folder of all three, and I should no longer need to worry about bouncing back and forth between projects to `yarn upgrade` in an _ALMOST_ circular loop to update these dependencies.

This is what's known as a _monorepo_, a singular repository housing the source code for multiple projects that share... something in common. What they share in common isn't always as cut-and-dry as my use case; for example, the repository for Ruby on Rails houses the source code for a lot of its own dependencies like Action Pack and Active Record that could, _strictly_ work independently. Vue CLI and Nuxt TNT do not work independently; it's sort of the inverse of the Rails project. Rather than Rails depending on all of these independent parts, TNT has these independent parts which depend on it.

The reason that monorepos have surged in popularity, I think, is because it makes development a lot easier. When you have these projects which depend on one another in whatever direction, and you have a team managing the development of all of them, it helps to have that all in one place. That way if an issue is identified with one project, but it's discovered that it originates somewhere else in the stack, it is trivial to address and push the common changes together.

There are tradeoffs, of course... One small example pertaining to open source development: someone may want to fork only Nuxt TNT and make a change to one part of that for their own purposes, but now they need to fork a monorepo and... figure out how their project includes a dependency coming from that form of architecture. So monorepos can be unfriendly. It's possible we'll address this a ways down the line by making each subdirectory a Git Submodule... but this is unfriendly in its own sort of way. We just aren't going to worry about that just yet.

For the time being, we just want to bring together TNT, Vue CLI TNT and Nuxt TNT in to one repository so that they can share code, share development, share common build tasks. And err...

There's an easy way to do that. We just copy the files and directories from each project into their desired locations in the main project. Easy peasy...

...but we lose each project's respective Git history that way. Preferably we would be... merging the three projects into one, revision history included. So let's figure out how to do that instead, shall we?

Ultimately, we want the resultant project to live in the same place that the core TNT library currently does (that's https://github.com/thombruce/tnt), and we want it to incorporate that core library as well as Nuxt TNT and Vue CLI Plugin TNT, each in three separate directories.

In other words, the base directory - `/` - is a sort of brand new space housing all three. And the three of them have their own directories - `/tnt`, `/nuxt`, `/cli` - somehow having preserved their respective histories from the Git repositories that currently house them...

Sounds... complicated enough.

We'll start by creating a clean project to house the three existing ones...

```sh
mkdir tnt-monorepo
cd tnt-monorepo
git init
touch .monorepo
git add .
git commit -m "Initial commit"
```

We also need something to be committed initially, hence the `touch .monorepo` command. This creates an empty file for me to commit - it can be anything, but it mustn't have a name conflict with any of the files we'll be merging, so I've just called it `.monorepo`.

Now we can do our first project merge...

```sh
git remote add -f tnt https://github.com/thombruce/tnt.git
git merge -s ours --no-commit tnt/main --allow-unrelated-histories
git read-tree --prefix=tnt/ -u tnt/main
git commit -m "Merge TNT Core"
```

...and that's the full merge. TNT's history is preserved and now lives in a `/tnt` folder in this new Git repository. There's a little to explain here, so we'll go through it step by step.

```sh
git remote add -f tnt https://github.com/thombruce/tnt.git
```

_This adds the existing GitHub repository for TNT as a remote repository for this one. A remote is essentially... some other Git repository we want to communicate with; typically it is a remote copy of the one we're actually working on, and it is often called 'origin'. In this case it's another repository entirely. We add the `-f` flag to tell it to fetch the Git history immediately, but we don't actually make any alterations to the state of our new monorepo at this point._

```sh
git merge -s ours --no-commit tnt/main --allow-unrelated-histories
```

_This is the command that actually merges the other repository's main branch. Often `merge` is used to... merge branches, and this is no different. The `-s ours` flag tells it to resolve conflicts by favouring the monorepo's files (this is "our" project). `--no-commit` halts the merge process before committing it, because these files are otherwise going to be merged into the root directory. And `--allow-unrelated-histories` is important to address the fact that our repository and TNT's have very different Git histories; Git typically disallows this merge for our own safety, but we have the option to override that. Next..._

```sh
git read-tree --prefix=tnt/ -u tnt/main
```

_This command reads the contents of the tree at `tnt/main`, doing so under the given prefix and, with the `-u` flag, updates the working tree with those contents. In other words... it moves the contents of the origin and branch at `tnt/main` into our new `tnt/` directory._

_And finally, we complete the merge and give it a commit message:_

```sh
git commit -m "Merge TNT Core"
```

Got it? ... Good, because I'm not sure I do. But it works!

Now we just repeat the process for the other two repositories...

```sh
git remote add -f cli https://github.com/thombruce/vue-cli-plugin-tnt.git
git merge -s ours --no-commit cli/main --allow-unrelated-histories
git read-tree --prefix=cli/ -u cli/main
git commit -m "Merge TNT CLI"

git remote add -f nuxt https://github.com/thombruce/nuxt-tnt.git
git merge -s ours --no-commit nuxt/main --allow-unrelated-histories
git read-tree --prefix=nuxt/ -u nuxt/main
git commit -m "Merge Nuxt TNT"
```

...and just like that, we have our full monorepo. Oh, and we can delete that initial file now...

```sh
git rm .monorepo
git commit -m "Cleanup: Delete .monorepo file"
```

Now my monorepo only houses those three directories and their project files. And I can be confident that the three projects still work independently, since nothing has fundamentally changed here. They'll each need `yarn install` to be ran, but they'll work. It's a working monorepo, and I believe this will let me better manage development of the whole TNT project going forwards.