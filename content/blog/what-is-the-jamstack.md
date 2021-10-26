---
title: What is the Jamstack?
description: 'A brief introduction to the Jamstack.'
authors:
  - Thom Bruce
date: 2021-06-06T00:41:11Z
categories:
  - Jamstack
tags:
  - JavaScript
  - API
  - Markup
---

_UPDATE: This post was originally written when I called this site Jameater and dedicated it to the Jamstack. I've since decided to broaden the subject material of this website to include all of my programming work, a lot of it but not all being Jamstack related._

Greetings curious parties and welcome to Jameater. I'm Thom Bruce, the author of this here blog and an ever-greater-and-greater advocate of the Jamstack. But what is the Jamstack, what's a Jameater and why am I writing about this? Well...

## What is the Jamstack?

The Jamstack (also stylised as JAMStack) is an approach to website architecture that emphasises the use of JavaScript, server-based APIs and markup, hence the name: **J**avaScript + **A**PI + **M**arkup = **JAM**Stack. Not every Jamstack website uses all three, but in principle all three are all a Jamstack website would ever use. To illustrate, let's consider _"baby's first website"_.

_"Baby's first website"_ is that first website that every web designer and developer ever produces. I threw my first website together when I was 11 with a _teach yourself HTML_ book I'd picked up at a Scholastic Book Fair. It didn't actually use any JavaScript, and it certainly didn't use any APIs, but I would make the case that this - a site comprised of purely HTML markup and a bit of CSS - was a Jamstack website. Which kinda means... the 'M' in 'JAMStack' is sort of... _the old way_ of doing websites. Most modern websites generate their HTML on a server using back-end code; this is how WordPress works, how Drupal and Joomla work, it's how Ruby on Rails works. It's... _the modern approach_. But a lot of the time, this _modern approach_ is unnecessary. A lot of sites that are running from servers, don't actually need those servers; they could be built on the Jamstack.

So, how does this work? Well you take _"baby's first website"_, for example, and you decide you want to add a little user interactivity - let them change the colours, say, or add a comment or a like. The way to do that without having a server to compute the change is to do it right in the user's browser; for that, we use JavaScript. JavaScript is a programming language understood by browsers, making it actually the most commonly used programming language in the world. It doesn't matter if your website is written on the backend in Ruby or PHP, or put together in WordPress, or it's just _"baby's first website"_ made of pure markup, JavaScript adds that frontend interaction that we've all grown accustomed to. _I should caveat this: A lot of interactivity can also be achieved with CSS alone, and where that's the case it usually should be, but JavaScript allows for plenty more advanced use cases._

And finally, there's the A in JAM - APIs. Because you don't need to run _"baby's first website"_ on a server just to be able to add comments or likes to _baby's_ posts. You just need access to a smaller, more dedicated server that handles user interactions. So you take the markup you've already got (which you can serve online without a server), you add a little JavaScript that says "when the user hits this button, send this information", and you send that information to an API that is dedicated to handling that sort of information. Et voila, _baby_ just deployed to the full Jamstack!

I'll discuss further concepts in later posts, like Static Site Generators are a pretty important concept; instead of writing page after page of HTML markup, an SSG will build the markup for your pages for you from templates and components that you customise. An SSG does a lot of the stuff that a WordPress server would to render a page, but whereas WordPress would go through this process on every single user request, an SSG only does this once. You build once and deploy to a serverless architecture, serving up what's known as a _"static site"_. These static sites become dynamic by use of JavaScript and APIs, but that's... precisely the same way that all websites feel modern and dynamic. Can you imagine if Facebook loaded a new page every time you wanted to leave a "like" on something? That's interactivity without JavaScript and APIs; it's page-loads, yeuch!

A quick and easy way to get started with Static Site Generators, if you're interested, is [Jekyll](https://jekyllrb.com/). I no longer use this myself (this blog uses [Nuxt.js](https://nuxtjs.org/)), but I do think Jekyll wins out for ease-of-use.

## What is a Jameater?

A derogatory term for someone from either one of two local towns, Whitehaven or Workington, who eats jam sandwiches (jelly sandwiches, if you're American). I'm [not joking](https://en.wikipedia.org/wiki/Whitehaven#%22Jam_eater%22). Anyway, I'm 'aving it for my Jamstack blog. I'm a proud Jameater, dammit!

## Why Write This?

_Can't answer right now; crying into my raspberry jam sandwich._ 😭