---
title: My First Nuxt Module
description: 'My first Vue plugin that integrates seamlessly as a Nuxt module.'
authors:
  - Thom Bruce
date: 2021-05-23T01:06:56Z
categories:
  - Jamstack
series:
  - Getting Started
tags:
  - Vue.js
  - Nuxt.js
---

In putting my blog together, and porting some old code to this new ecosystem, I began to create a... well not overly but still a bit of a cluttered project. I brought over some code for parsing screenplays and rewrote some old code for displaying different kinds of clock - a project I originally wrote in Ruby on Rails almost a decade ago. The obvious thing I still needed to do was to clean this up by creating my own Nuxt modules.

Problem: Creating a Nuxt module isn't super well documented just yet. I found [this entry on the official Nuxt.js blog](https://nuxtjs.org/blog/improve-your-developer-experience-with-nuxt-components/#third-party-component-library) and [this example project](https://github.com/Krutie/Kru-Components) which it links to. The actual part about developing a third party module and the example project, however, are towards the bottom of the blog entry. It's otherwise mostly just about Nuxt components and how to use those in your project. Still, it was the guide I needed. So, how did I create my first (and second) Nuxt module? Well...

First, I made the directory for my project (`mkdir`), changed directory into it (`cd`), initialised it as a Git repository (`git init`) and as a JavaScript package (`yarn init`):

```sh
mkdir vue-timepiece
cd vue-timepiece
git init
yarn init
```

`yarn init` is an interactive process, asking for the name, version and some other details about the package being created. The result is to create a `package.json` with those values. _Why am I explaining this in so much detail? This isn't a 'My First JavaScript Package' post..._

Anyway, next I moved my existing Vue components across from my Nuxt project to this new directory, placing them in the `src/components` directory. Nothing special about that, the components did not change in the transfer; they're just in a new folder in a different project.

Now, I could stop there. I could push that up to the NPM package registry, install it in my Nuxt project and manually `import` my components as one might in a standard Vue project. But Nuxt offers us a little magic, yet.

I added a file specifically for use by Nuxt at `nuxt/index.js` that extends the global components option so that I can use my components anywhere without having to import them.

```js[nuxt/index.js]
import { join } from "path"

export default function () {
  const { nuxt } = this

  // Make sure components is enabled
  if (!nuxt.options.components) {
    throw new Error('please set `components: true` inside `nuxt.config` and ensure using `nuxt >= 2.13.0`')
  }

  this.nuxt.hook("components:dirs", (dirs) => {
    // Add ./components dir to the list
    dirs.push({
      path: join(__dirname, "../src/components"),
      prefix: "timepiece"
    })
  })
}
```

 This has the added benefit of working seamlessly with the Nuxt Content module so I can use my Timepiece components directly from inside the Markdown of my blog posts, like this:

<div class='w-min mx-auto my-5'>
  <timepiece-analog></timepiece-analog>
</div>

It's this easy:

```md
<timepiece-analog></timepiece-analog>
```

Nuxt modules and the `components:dirs` hook are like magic! In fact, I've already gone ahead and ported my screenplay parser over to a Nuxt module too, so I can also share formatted screenplay snippets like this one:

<fountain-screenplay>
<template>
INT. MUSIC ROOM - EVENING

CASSANDRA (CONT'D)
(sign language)
What are we going to do?

David pulls a leaf of paper from his pocket.

He flattens it out onto the music sheet holder of the piano.

DAVID
(sign language)
Ben's work. He's a little genius.

Ben's drawing displays David sketching and Cassandra playing the piano.

DAVID (CONT'D)
(sign language)
We'll put on a proper show for you.
</template>
</fountain-screenplay>

It's like magic, I tells ya!
