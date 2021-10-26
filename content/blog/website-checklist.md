---
title: Website Checklist
description: 'My checklist for everything that ThomBruce.com needs or should have.'
authors:
  - Thom Bruce
date: 2021-05-12T12:43:03Z
categories:
  - Jamstack
series:
  - Getting Started
tags:
  - Web Development
  - Hosting
  - Nuxt.js
  - Netlify
  - UptimeRobot
  - Upptime
---

- [x] Domain

I purchased my domain, thombruce.com, from [Namecheap](https://www.namecheap.com/).

- [x] DNS

The DNS at Namecheap wasn't stellar, so I hooked up my domain's DNS with [Cloudflare](https://www.cloudflare.com/).

- [x] Basic Website

Nuxt.js

- [x] Source control

GitHub/GitLab

- [x] Web hosting

ThomBruce.com is being hosted by Netlify.

- [ ] Continuous Delivery

At this time, we're using Netlify's 300 free build minutes, but Nuxt is kind of demanding so these get eaten up kind of fast. GitHub and Circle CI (for projects hosted on GitHub) offer unlimited build minutes for open source projects, buuuut... GitHub only allows 1GB of large file storage, after which I would have to consider a new CDN (see below).

- [x] Uptime Monitoring and Status Page

Formerly, I've monitored my websites using [UptimeRobot](https://uptimerobot.com/). My new status page at [status.thombruce.com](https://status.thombruce.com/) uses [Upptime](https://upptime.js.org/).

- [x] Content

Nuxt Content

- [ ] Content Management System

At present I'm just using a code editor, Visual Studio Code, but I have plans to develop a separate CMS.

- [ ] Content Delivery Network (CDN)

Currently, images are being stored directly in the repo with GitLab's generous 10GB of free LFS storage. They are mutated directly by the application and served by Netlify's CDN - Netlify provide 100GB bandwidth per month for free across all sites per account. But I am looking at moving to Cloudinary, which offers 25GB of storage or 25GB of viewing bandwidth or 25k transformations, or any combination of those adding up to 25 credits. I do not know if use of Nuxt Image would alleviate the demand on Cloudinary by storing resultant transformations locally on Netlify's CDN... but it is certainly something to look at. The best option is always to store only what is needed.

- [x] Site Interactivity (Comments, etc.)

I'd like to add some basic interactivity to ThomBruce.com. For this, I'm thinking of installing [Staticman](https://staticman.net/). Staticman would allow users to add comments; they'd take a minute to appear as live on the website because Staticman would send them to the static site generator, so this wouldn't do for a live chat or anything like that, but for comments, maybe a guestbook? It could work. That's comments covered by Staticman, but it leaves a bit to be desired and certain interactions will be impossible with that as the backend - this remains to be looked at further.