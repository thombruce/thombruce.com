---
title: How I Build My Websites?
description: 'What goes into one of my static sites in 2021.'
authors:
  - Thom Bruce
date: 2021-06-21T22:09:01Z
categories:
  - Jamstack
tags:
  - Nuxt.js
  - TNT
---

For the past... I don't actually know how many years, I have been trying to pursue a certain ideal of _Convention over Configuration_. This is the web development philosophy that advocates for having a sort of set of best practices that are repeatable, reproducible, and that emphasise essentially... lazy programming. I am a lazy programmer. It's a good thing, actually. The thing about lazy programmers is not that we're "lazy" in the classical sense. We actually work very hard in the present in order to reduce work in the future thanks to having a set of reproducible tools, packages and practises. _Conventions_, that is, that simply drop-in and require little to know _configuration_. As I say, I've been at this for I don't know how many years so it's an ideal that's always changing, my conventions are always evolving. Lately I've been pursuing this on the [Jamstack](/categories/jamstack), and in particular in Nuxt.js.

Before I divert to far off the course here, the reason I'm writing this blog post is to explore - step by step - the conventions I've now established for myself as a Nuxt.js developer. I want to do that by detailing below the process as I actually put another site live, from scratch. So I'm going to start with no website, and by the end of this article have a brand new website live on the internet. It won't take very long, because I'm very confident in these conventions I've established. Along the way, I may add a little commentary about improving this process or about where I want to change things in the future. Okay, let's begin. The website I'm going to be building is just called 'Screenplays', and it is essentially just a blogging website like this one, except that it will be built to specifically host some screenplays I wrote back in university. Here we go...

## Creating the Project

First up, I'm running what has become a standard for creating Nuxt apps. `yarn create` accepts a template argument - `nuxt-app` in this case - and a name for the project which is going to be `screenplays`.

```sh
yarn create nuxt-app screenplays
```

This runs an interactive process, meaning it will ask me a few questions about the setup and what features I desire:

```sh
Project name: screenplays
Programming language: JavaScript
Package manager: Yarn
UI Framework: None
Nuxt.js modules: Axios, Progressive Web App, Content
Linting tools: None
Testing framework: None
Rendering mode: Universal
Deployment target: Static
Development tools: jsconfig.json, Dependabot
Version control system: Git
```

Key to my setup, I've selected no UI framework, the universal rendering mode and the static deployment target.

The universal rendering mode essentially says "this Nuxt app should be built as if it is to be rendered by a server", but that's not actually the case (I'm oversimplifying); the static deployment target effectively adds to that "and the server rendering it is the build process". The result of those arguments is that this builds and generates a "static site", which is a set of HTML files not actually requiring a server to deploy for public visitation.

The reason I've selected no UI framework isn't actually because none of them interest me - in fact, one of the listed options is Tailwind CSS, which I will be using - but that I've configured everything I would like from a UI framework in a package of my own that I will install momentarily.

## Style and Structure

The default Nuxt template, even without a style framework, comes with some pages and components that I'll just want to delete. These are:

- components/Logo.vue
- content/hello.md
- layouts/default.vue
- pages/index.vue

_I should delete or replace some stuff in the static directory too, but this is a process in progress._

Next, I'll add my own package, TNT (it doesn't really stand for anything but is derived from the idea that it's _Thom's Nuxt Template_):

```sh
yarn add https://github.com/thombruce/nuxt-tnt --dev
```

And I'll install the module by registering it as a buildModule in nuxt.config.js:

```js[nuxt.config.js]
buildModules: [
  '@thombruce/nuxt-tnt'
],
```

This installs some custom components, all registered and globally available, it installs Tailwind CSS with a default configuration setup to my liking, it even registers a couple of custom layouts (which is why I was able to delete the default layout above). The one thing it does not install that I wish it did is a set of default pages. This isn't currently possible in Nuxt, but should be in Nuxt version 3 which is currently in active development.

So, rather than write my own pages, I copy them across from one of my existing projects. In this case, I've copied them from this website into my project. They're specifically designed to be reusable like this. By default they are:

```
pages
├── _taxonomy
│   └── _term.vue
├── blog
│   ├── _slug.vue
│   └── index.vue
├── _page.vue
└── index.vue
```
`_page` and `_taxonomy` are pretty special. `_page` is polymorphic and will act like page with a dynamic slug at the root of the site _or_ as a taxonomy index; the taxonomy slug can also be dynamic. This saves me having to create very similar pages for tags, categories, series... that kind of thing. In the case of my screenplays project, I don't actually want the blog pages, so I'll rename that directory to 'screenplays' and replace the two occurrences of the word 'blog' in those files with that as well. Those two occurrences actually pertain to the content directory, so with that change made...

## Adding My Content

Like I said, this is a site for some screenplays I wrote back in university. Back then I used Final Draft and Celtx, two leading script writing platforms with proprietary formats where that format is also... binary. But I've converted all of them to the open source Fountain format more recently, which is a text file format like Markdown (based on Markdown in fact). I've also written another package specifically for handling that file format in Nuxt Content, so I'm just going to copy those files into `content/screenplays` and install my `vue-fountain` package:

```sh
yarn add @thombruce/vue-fountain --dev
```

```js[nuxt.config.js]
buildModules: [
  '@thombruce/nuxt-tnt',
  '@thombruce/vue-fountain/nuxt'
],
```

This is an extra step I wouldn't normally have to take if it were a standard, markdown-based static blogging site, but it is a step I do take here for my screenplays. But in fact... that's it.

Now having those content files in place, I can navigate to them when I boot up the dev site with `yarn dev`. Those pages are generated and exist. If it were a more straightforward blogging site, I'd just create Markdown files and be rolling.

This site is ready to deploy!

## Ready for Launch!

I could at this point simply run `yarn generate`, which would build my website and put all of the files I need into a `dist` folder. Then I could drag and drop that directory into a hosting service like Netlify.com and _just like that_ my site would be live. But I want a couple of extra things. In consideration of hosting a website, I want:

- Source control
- Continuous integration
- Hosting

The hosting part we've just discussed. I will in fact be using Netlify, but I won't be dragging and dropping the dist folder into their deploy window. In order to achieve the criteria of a source controlled site with continuous integration, I'll also be using GitHub and GitHub Actions. Those familiar with Netlify might ask, _why GitHub Actions instead of Netlify's automated deployments?_ A fair question. I would in fact prefer to use Netlify's automated deployments, because their service is fantastic, but I want to benefit from GitHub's unlimited free build minutes for open source projects.

To achieve this, I need a GitHub Actions Workflow for which I again have a standard template that I'll copy and paste:

```yml[.github/workflows/deploy.yml]
name: deploy

on:
  push:
    branches:
      - main

jobs:
  deploy:
    runs-on: ${{ matrix.os }}

    strategy:
      matrix:
        os: [ubuntu-latest]
        node: [14]

    steps:
      - name: Checkout
        uses: actions/checkout@master
      # https://github.com/actions/checkout/issues/165#issuecomment-657673315
      - name: Create LFS file list
        run: git lfs ls-files -l | cut -d' ' -f1 | sort > .lfs-assets-id
      - name: Restore LFS cache
        uses: actions/cache@v2
        id: lfs-cache
        with:
          path: .git/lfs
          key: ${{ runner.os }}-lfs-${{ hashFiles('.lfs-assets-id') }}-v1
      - name: Git LFS Pull
        run: git lfs pull

      - name: Setup node env
        uses: actions/setup-node@v2.1.2
        with:
          node-version: ${{ matrix.node }}

      - name: Install dependencies
        run: yarn

      - name: Generate
        run: yarn run generate

      - name: Deploy
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_branch: dist
          publish_dir: ./dist
```

Now, when I push to GitHub, this workflow will build the site and commit the result to a separate branch called `dist`.

At this point I can create a new repository on GitHub and:

```sh
git add . && git commit -m "Ready to launch!"
git remote add origin https://github.com/thombruce/screenplays.git
git push -u origin main
```

When the workflow is complete, I can create a new site on Netlify linked to the repo and publish straight from the `dist` branch.

Done!

## Wrapping Up

Okay, so if I weren't also typing this up this process would have taken me what... about twenty minutes? No more, certainly. That's largely thanks to having so much of my personal configuration written into my TNT module for reuse. There are some problems and some things I left out. For instance... I've also had to add a `tnt.config.js` file and slightly modify `nuxt.config.js` to import it; ideally TNT would handle that import itself, and even more ideally it would have some fallback configuration that would be used if the file didn't exist. That's a chore for me yet to take on, but shouldn't be too tricky, hence I left it out of my write-up because I don't know that it will remain relevant for long. I've also had to do some other specific customisations that were specific to this site handling screenplays; those aren't typically relevant to a site build, so I omitted them. I also still need to install Git LFS - something I typically do to handle things like image files - and do some of my own style customisations for the site, mainly by adding a `tailwind.config.js` file and modifying some configuration there.

As I mentioned when copying across page templates, a Nuxt module like TNT cannot currently offer a set of custom pages but this is supposed to become possible with Nuxt 3. Strictly it is achievable by injecting some custom routes... but this doesn't result with an easily overwritable behaviour. That's one of the perks of TNT - any of its components can be overwritten simply by creating your own component existing at the same path in your project directory. I want that for pages too, and so I'm eagerly awaiting Nuxt 3 at which point I think I'll be able to "finish" TNT to my liking and maybe make it more useful to others too.

In less than the time it took me to write those two paragraphs of wrapping up, my GitHub workflow was complete so I have:

1. Created a site on Netlify from the `dist` branch of my GitHub repo
2. Added a custom domain to my DNS records pointing at that site

This took less than a minute. It'll take a little longer for the DNS to propagate and for Netlify to provision an SSL certificate, though this is typically pretty quick too. By the time anyone reads this, the site should be available at [ink.thombruce.com](https://ink.thombruce.com/).

That's a basic blogging website done in under twenty minutes. The template has a minimalist kinda style which I still need to work on (I'm more developer than designer), and there are some other kinks to work out yet too, but that's... that's it. I have this process down to a tee. This is what proactive laziness looks like. 😅