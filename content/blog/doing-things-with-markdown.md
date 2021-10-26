---
title: Doing Things with Markdown
description: 'Exploring and extending Markdown''s capabilities'
authors:
  - Thom Bruce
date: 2021-05-20T14:05:39Z
categories:
  - Jamstack
series:
  - Getting Started
tags:
  - Blogging
  - Nuxt.js
  - Markdown
---

_UPDATE: This blog post was originally written at ThomBruce.com, before I decided to separate and dedicate specific domains to singular subjects (code in this case). Some of the markup below is broken as a result. For instance, I do not know if I have use for KaTeX outside of this blog post so I have not installed it._

I want to go over again what's possible with Markdown; in particular what is possible with Nuxt's markdown parser which I've learned is slightly extended, and what I would like to add myself.

## Typography

I've mentioned [elsewhere](/blog/no-going-back) what _is_ and _isn't_ possible in **Markdown** natively in terms of typography, and what can be achieved using HTML instead. Just to recover it here, this short paragraph has made use of a title, a link, italic and bold text. It was written like this:

```md
## Typography

I've mentioned [elsewhere](/blog/no-going-back) what _is_ and _isn't_ possible in **Markdown** natively in terms of typography, and what can be achieved using HTML instead. Just to recover it here, this short paragraph has made use of a title, a link, italic and bold text. It was written like this:
```

## Inline Code & Code Blocks

Let's cover code blocks quickly since I've just shown one off. There are a few ways to write these in Markdown, but most commonly used are the three back-ticks, like so: `` ``` ``. These are placed above and below the block of code we want to enclose. The top set also accepts a language argument: `` ```js ``. Nuxt's parser extends the standard with support for a file-name attribute too: `` ```js[myFile.js] ``. Putting all of that together, we get a code block:

```js[myFile.js]
import otherFile from './otherFile.js'

export default {
  hello: 'world',
}
```

## Images

I've already partially discussed ways to include images in Markdown [here](/blog/images-and-nuxt). I'm still, for the time being, using the Nuxt Image component for the benefits that offers.

## Tables

Here's a table I've just copied over from [another page](code/thom-bruce-com):

| Domain | DNS | Framework | Source | Hosting | CI/CD | Status | Content API | CMS | CDN | Comments |
| --- | ---|  --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [Namecheap](https://namecheap.com/) | [Cloudflare](https://cloudflare.com/) | [NuxtJS](https://nuxtjs.org/), [Vuetify](https://vuetifyjs.com/) | [GitLab](https://gitlab.com/) | [Netlify](https://netlify.com/) | [Netlify](https://netlify.com/) | [Upptime](https://status.thombruce.com/) | Static | File System | [GitLab](https://gitlab.com/), [Netlify](https://netlify.com/) | [Staticman](https://staticman.net/)

The Markdown for that looks like this:

```md
| Domain | DNS | Framework | Source | Hosting | CI/CD | Status | Content API | CMS | CDN | Comments |
| --- | ---|  --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [Namecheap](https://namecheap.com/) | [Cloudflare](https://cloudflare.com/) | [NuxtJS](https://nuxtjs.org/), [Vuetify](https://vuetifyjs.com/) | [GitLab](https://gitlab.com/) | [Netlify](https://netlify.com/) | [Netlify](https://netlify.com/) | [Upptime](https://status.thombruce.com/) | Static | File System | [GitLab](https://gitlab.com/), [Netlify](https://netlify.com/) | [Staticman](https://staticman.net/)
```

I haven't done much to style it besides ensuring that it remains readable with overflow scrolling.

## Nuxt Components

Not native to Markdown, Nuxt has support for dropping in components, like this Media Card I made:

<!-- WONTFIX: <media-card item='/media/movies/spider-man-no-way-home'></media-card> -->

```md
<media-card item='/media/movies/spider-man-no-way-home'></media-card>
```

There's nothing more to it than that. I just create and modify the Media Card component as part of my code, and if I change it then the changes will be automatically applied here too. I can drop in other components too, like a map:

<!-- WONTFIX: <map-view place='carlisle'></map-view> -->

```md
<map-view place='carlisle'></map-view>
```

They could get a little verbose if I wanted to do something particularly complicated, but with drop-in components like these it's possible to add absolutely anything I want to a Markdown document.

## Markdown Extensions

There's at least one other Markdown extension active by default. That is checklists like this one:

- [x] One
- [x] Two
- [ ] Three

```md
- [x] One
- [x] Two
- [ ] Three
```

That can come in pretty useful.

As well as this, I need to make note that Nuxt uses Remark under the hood which, along with Rehype, supports adding additional plugins. It's documented in Nuxt Content [here](https://content.nuxtjs.org/configuration#markdown).

One plugin I am eager to install (so will do right now) is the [remark-math](https://github.com/remarkjs/remark-math) plugin. To install, it seems I need to `yarn add remark-math rehype-katex` and then modify nuxt.config.js like so:

```js[nuxt.config.js]
  content: {
    markdown: {
      remarkPlugins: [
        'remark-math'
      ],
      rehypePlugins: [
        'rehype-katex'
      ]
    }
  },
```

After which, I should be able to add inline math ($N$) as well as math blocks like this one:

$$
N = R_* \cdot f_\mathrm{p} \cdot n_\mathrm{e} \cdot f_\mathrm{l} \cdot f_\mathrm{i} \cdot f_\mathrm{c} \cdot L
$$

How much use I'll get out of that, I don't know. I've considered getting back into mathematics a little bit, so we'll see.