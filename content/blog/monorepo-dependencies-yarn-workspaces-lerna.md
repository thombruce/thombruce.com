---
title: Managing Monorepo Dependencies with Yarn Workspaces and Lerna
description: 'Configuring my monorepo for easier development with Yarn Workspaces and Lerna'
authors:
  - Thom Bruce
date: 2021-09-06T20:36:44Z
categories:
  - Journal
series:
  - My Process
tags:
  - Yarn
  - Monorepo
  - Lerna
---

In [my last post](/blog/tnt-monorepo), I talked about taking my three TNT projects - those being the core Vue package, plus a version packaged for Vue CLI and one for Nuxt.js - and creating, from them, a monorepo to house all three. This, I said, would make development a lot easier. It does. A caveat I didn't mention is that it could make dependency relationships harder to manage... which it, err... does, sort of. In some respects, dependencies are easy to manage between the three projects, and in other respects it is a little more difficult to have the three communicate with one another. But not to worry; that's where tools like [Yarn Workspaces](https://classic.yarnpkg.com/en/docs/workspaces/) and [Lerna](https://github.com/lerna/lerna) come in. The first of these, Yarn Workspaces, is a way to share dependencies between the three projects as well as having each one depend on the others during development. The second, Lerna, does that same thing as well in a different way, but on top of that it also offers some version management and publishing tools.

So, what's the benefit and what's the difference? Well, the main feature of both Yarn Workspaces and Lerna is multi-package dependency management. With Yarn Workspaces active, for instance, one only has to run `yarn install` from the base directory and Yarn will iterate through all of the packages and install all of their dependencies. It installs all dependencies in the project root as well, so that the packages all share their common dependencies, significantly lowering the install time and size. And it symlinks the other packages in the workspace, so that the local, development version of each package is used in development. All of this, Lerna does as well but slightly differently. And Lerna adds the capability to run scripts across all packages, manage semantic versioning across them all, and publish all of them to the NPM registry with a single command. So then... if Lerna does the same and more, why have I chosen to use both?

Good question. I actually did experiment with using Lerna alone, but it doesn't play nicely with Yarn's package management which I was already using. It's also had a series of developments that seem to have outpaced documentation - that is to say, it isn't poorly documented, but the documentation is somewhat incomplete and in need of some clarification. Aside from this, Yarn Workspaces is a native solution which is typically something you can depend on. It just isn't as feature rich as Lerna, so... we're lucky that Lerna has this under consideration and can be configured to work with Yarn Workspaces instead of its own dependency management solution. The resultant setup is something I absolutely adore, and am sure to use again in the future. Let's examine it...

## The Setup

The only `package.json` configuration required to work with Yarn Workspaces is... _well, actually, it's this:_

```json
"workspaces": [
  "packages/*"
]
```

That's an absolute minimum. The `packages/*` glob pattern can be anything; this one just says, treat every folder inside of the packages directory as a separate package. _NOTE: Yes, I've also moved my project folders yet again into a `packages/` directory - it's a good pattern._

Yarn Workspaces also used to require that the root package be marked private with `"private": true`. In fact, later docs don't suggest that this is at all deprecated, just that they've removed the requirement to make migrating to this approach a little easier for package developers. In other words, it is still best practise.

With that in mind, I moved my various packages into the `packages/` directory, and I added the following to a root `package.json`:

```json
"private": true,
"workspaces": [
  "packages/*",
  "docs"
]
```

You can see I've also added `docs` to the workspaces patterns; I decided to keep the documentation website separate, as it isn't strictly one of the packages I'm aiming to publish.

With that done, using Yarn Workspaces is as easy as running `yarn install`. Dependencies are then installed in the root directory and the packages depending on each other get symlinked. Easy peasy!

Next, I installed Lerna...

```sh
yarn add lerna --dev
```

...and added a basic `lerna.json` configuration file:

```json
{
  "version": "0.1.0",
  "npmClient": "yarn",
  "useWorkspaces": true
}
```

Typically, the `lerna.json` file would have a `packages` value containing a similar list of globs to the one in `workspaces` above. Since we're telling it that our `npmClient` is `yarn` though, and that we want to `useWorkspaces`, we can omit the packages value. Because `useWorkspaces` is `true`, Lerna will instead use the `workspaces` array in our main `package.json`.

And... Oh wow, that's really all it takes.

So I can now run commands like `lerna run build` to run the `build` command in all packages that have that command, and `lerna publish` them all in one go and increment their versions. Monorepo management on autopilot!

In fact, I love this setup so much I've already started wondering if it's possible to adopt for some of my other projects. I'm sure it is. Either way, I'm gonna be adopting this structure a lot going forwards.