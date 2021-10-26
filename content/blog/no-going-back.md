---
title: No Going Back
description: 'On restarting my blog from scratch every few months, and not wanting to do that again.'
authors:
  - Thom Bruce
date: 2021-04-19T08:15:10Z
categories:
  - Jamstack
series:
  - Getting Started
tags:
  - Blogging
  - Markdown
  - Nuxt.js
---

Hi, and welcome to the one-thousand-three-hundred-and-thirty-seventh iteration of my blog. Okay, granted, that's a little bit of an exaggeration, but I really do have a _starting over_ problem. I keep on beginning new versions of my blog as I decide that I want to move to a new technology or I want to organise things a little differently. The first blog I actually wrote was a very long time ago; it was on MySpace, if you remember that website - a social platform from the _Before Facebook_ times. As I became more and more interested in making websites, I moved to WordPress, then I considered more sophisticated CMS platforms like Joomla. Eventually, I became a full-fledged web developer writing my own platforms in Ruby on Rails. Each step of this progression offered me more and more freedom and control; control to have my website behave a certain way, to do more technically difficult but really quite interesting things. And now... Now I've taken a few steps backwards.

See, this blog post is being written in a plain text editor. Think... Microsoft Word, but if you couldn't add titles, bold text, italic, or any other formatting. It is a plain text document. That doesn't actually mean I can't do things like add titles and **bold** or _italic text_. I can, as you can probably see there, but they are being written as a form of markup in a Markdown document. As I see it presently, that previous sentence looks like this:

```md
That doesn't actually mean I can't do things like add titles and **bold** or _italic text_.
```

Note the `**` enclosing tags to denote **bold text** and the `_` tags to denote _italic_. I like Markdown a lot; nothing is hiding. When you write for the web, the result is typically going to be HTML. That sentence above might render something like this in the end:

```html
<span>That doesn't actually mean I can't do things like add titles and <strong>bold</strong> or <em>italic text</em>.</span>
```

If you use a CMS like WordPress, or if you're writing a document in Microsoft Word, there is a certain magical step that gets hidden. You write rich text, you see the **bolds** and the _italics_, but how it renders to a website is hidden from you. As a developer, I like to know that I have total control. I can use HTML to add <span style="color:red;">c</span><span style="color:orange;">o</span><span style="color:yellow;">l</span><span style="color:green;">o</span><span style="color:blue;">u</span><span style="color:violet;">r</span>, but HTML can be a little overly verbose as in the example above and... well, here's the HTML required to render those colours just now:

```html
<span style="color:red;">c</span><span style="color:orange;">o</span><span style="color:yellow;">l</span><span style="color:green;">o</span><span style="color:blue;">u</span><span style="color:violet;">r</span>
```

I prefer cleaner markup, not having to type out quite so much, but I like having that control. So Markdown is great for that. It is a superset of HTML, meaning I can include HTML (as I did to produce that colour effect), but for the most part I can write clean and simple, but still _dynamic_ text that remains easy to read.

This is one of the reasons I today prefer things like Static Site Generators over the server-dependent systems like WordPress, Joomla and Ruby on Rails. Strictly, I could write Markdown in any of those systems too, but there are other benefits to Static Site Generators. They don't depend on a server, so the websites they produce are highly dependable. They skip the step where the server _figures out_ how to render the HTML; by the time a user is visiting a website produced by a Static Site Generator, the HTML is ready to deliver, so they're also blazing fast and much less fault-prone. But they can still do fancy, programmatic stuff like... for example, converting Markdown to HTML, they just do it earlier than at runtime. The trade-off, you see, is that publishing isn't quite instantaneous as it might be on a server-supported website. You could never have a chat app, for instance, that used an SSG to communicate chat messages, because every new message would mean having to build the HTML and republish the website again; it would be too slow. For something like a blog though, or for documentation or informational websites, this trade-off is worth it for the gains. So what if publishing a new page takes a minute instead of a microsecond, we've saved a lot more time for every visitor to the website, and a lot of energy in terms of rendering HTML on a server.

Static sites won't replace everything, obviously, but we can still integrate dynamic features into static pages. This is why I've chosen [Nuxt.js](https://nuxtjs.org/) as my generator of choice; it's a very flexible JavaScript framework for developing sophisticated applications... that just so happens to also have static site generation built in. So... _kind of_ best of both worlds. Still might need a server to facilitate certain things, but it's a strong foundational start.

So, this is my new blog. And... who knows? Maybe in the future I will decide I need to move it elsewhere, but... I really want to try to remember this time, _No Going Back_. If I move, I can take these blog posts, these markdown files with me. If I need to host the content on a server instead, I can write code that uploads them for me. Heck, I could have written scripts that exported from WordPress or Rails and gave me a zipped up folder of Markdown files, but my old blogs are lost to time now, I think. I've just got to not let that happen again.

I don't know what you'll be seeing from me here, yet, but I will update as I go along and have anything worthwhile to say. Thanks for reading.