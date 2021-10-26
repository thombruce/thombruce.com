---
title: In Consideration of the Perfect Open Source Architecture for Web-based Apps
description: 'Open source software should be free for all, easy to develop and easy to deploy'
authors:
  - Thom Bruce
date: 2021-08-21T14:51:08Z
categories:
  - Journal
series:
  - My Process
tags:
---

In my blog post yesterday I declared my intention to make three web-apps - _Hosted_, _Marmalade_ and _Dynamite CMS_ - each of which would be built on, _probably_, Ruby on Rails. That part is... a little selfish. I mean it makes sense; Ruby is a beloved language with a lot of support and development in the open source community. But also... it's the language that my career is based on, and that's the main reason I'm pushing for it. Which again... makes sense. I've worked with Ruby for about ten years, so I of course should run with it as my experience will be of great benefit while managing such a project. That said, it doesn't couple with Nuxt as I'd hoped it might... and other backends like Node.js' Express do. I know for a fact I want what at least functions like a single-repository installation - one click deploys and all that good stuff; essentially something that behaves like a monorepo. If I really want to use Nuxt then... well, the choice should probably be Express. I can couple Vue, the framework Nuxt is based on, with Rails just fine but... do I lose the capability to offer a frontend only install? Say for anyone wanting to deploy their own frontend but using a hosted API? Or for packaging the project as a desktop or mobile app. That is possible with Rails, but a JavaScript Single Page Application (SPA) is probably preferred, right?

Mainly I'm muddling through all of these questions because I have a contradiction of wants: I want to use Rails, I want to use Nuxt. I can't use both unless... they're separate repositories, and then I lose some ease of installation. As a compromise, I can use Vue in place of Nuxt but I wouldn't have access to my own TNT project unless I modified it extensively... which I'm not ruling out, but that will be a later consideration. Okay, but let's try to reconcile this. What is my philosophy for open source application development? And how do I match that philosophy in application?

## The Philosophy

Okay, so these are hobby projects and I want them to be...

1. Free Software
2. Open Source
3. Easy to install
4. Easy to develop

Free doesn't necessarily mean absolutely free, but I do mean free as in both liberty and beer. Developers should be able to take the project, adapt it, redistribute it, and they should be able to run it without having to pay. There'll be some caveats to how _free_ it is dependent upon choice of _free software_ or _open source_ license, and running it at no cost will be possible but likely to run into hosting costs eventually - the point being there's no payment required to actually use the open source software. And it should be open source, meaning developers can read the source code, contribute to it, creating extensions of it. To ensure that it is easy to develop, the code should be well written, well documented and ultimately well tested. All of this is the easy part.

The hard part is ease of installation. I don't want any developer or other user to have to install several different parts just to get the application running. If they want to host and run the software, this should ideally be a one-click process. Similarly, if they want to run the frontend elsewhere and benefit from someone else's hosting (either paid or provided freely), this too should ideally be a one-click process with ideally only one or two details to configure (the host domain for the backend). Maybe it should have a default host, meaning a true one-click deploy but potentially giving favourable weight to my own hosted solution... or maybe it should ideally also run without requiring a backend. That isn't always possible, but a lot of applications could in fact work with just browser storage alone - this may work with one of the three I'm planning here.

So, it should be free to use, free to modify and redistribute, open to contributions with a clear process and good documentation, and it should install with minimal effort. How do we achieve that?

## The Application

In order to keep the software free (as in liberty and beer), we just need to choose the right license. As well as the right license, I'll also want to encourage donations and sponsorships so that continuous development can be justified without the end user having to pay anything to use the software. Broadly... there should be options. Like, I can't right now justify hosting an online application for any number of users at no cost to them; but I will always be able to justify making that software free for them to install and host themselves. Perhaps a ways down the line, we can have a paid service model that offers managed hosting, but I'll always keep the core, open source project free in the sense of both liberty and beer.

Ease of development is very much a case of keeping code well-written, well-documented and well-tested. In these efforts, we will benefit from the underlying tech stack being well-supported and loved by the community. I've pretty much settled on Ruby and Vue for the back and front end respectively - these both have large, supportive communities. Ease of development then is just a matter of ensuring that the code always remains clear, documented and tested. It also ties into whether or not the software is easy to install; ideally, a developer should be able to pull the project once and get it running without any unconventional changes in their setup. That is to say, if they have Ruby and they have Node installed, this should run on their system no problem. The rest of the stack, then, should be environment agnostic. _Maybe_ there are some configurations I do need to make, like favouring a certain database, but we'll aim to avoid this as much as possible.

What does that leave? Ease of installation, which is the same as ease of deployment really. And when I speak about deployment, I want it to mean to varied environments. For instance, to the web - that's the easy part - to desktop applications and to mobile. Now, I've deployed Vue apps as desktop and mobile apps in the past, but this - _I think_ - requires a little bit of decoupling from the Ruby on Rails backend. I see two applications here:

1. The Vue application, capable of running in browser or in a wrapper as either a desktop or mobile application.
2. The Ruby backend, which serves both an API as well as the Vue application if it is navigated to via HTTP.

And both of these should be runnable with a single-click installation, but importantly the Vue app that the Ruby backend does serve is the same as the first application.

So then... two projects? The Vue application itself, then the Ruby backend which includes that application as a dependency or perhaps as a submodule? ... Maybe. The problem with a dependency approach is that it does run counter to one ease of development principle; if a dev goes looking to modify the frontend but the frontend is an external dependency, suddenly they need to install and perhaps even fork an entirely separate application. I just need to decide whether this is two repos or just the one monorepo.

Honestly, didn't think I'd go this way but I am leaning towards trying to put this together as a monorepo. The frontend would sit in a dedicated directory, and the root `package.json` file could list all of the scripts and dependencies needed for both aspects of the project. However... I'm undecided and I think I will go with two separate repositories initially, if only because it's a pattern I've worked with before.

## In Summary

So in conclusion... We have a sort of checklist of considerations and answers to those considerations:

- Choose a license (I'll probably stick with MIT initially but change it up later if it makes sense)
- Keep it well documented, well tested and open
- Separate the front and backend, but relate via dependency for ease of installation of every aspect of the application
- Monorepo or separate repos? (We'll go with separate for the time being, but a monorepo does sound attractive - can change this later)

Gonna continue to detail the process as I build this out in future blog entries.