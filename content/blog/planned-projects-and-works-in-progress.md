---
title: Planned Projects and Works In Progress
description: 'Making a note here of all of the projects I want to start or continue working on.'
authors:
  - Thom Bruce
date: 2021-08-20T18:15:52Z
categories:
series:
tags:
---

Somewhere in this blog post I will talk about a project management tool that I've worked on in the past and intend to redo from scratch in the future... and damn, do I need that. I have this problem where I've just got all of these ideas on the go at once, on top of actual work and other responsibilities. It's hard to keep track. I just need to put them to paper, so to speak - write them down, keep a list and mark them off it as I go. Anyway, hi, this is that list. These are some of the projects that I'm either currently working on or thinking about working on soon.

## TNT

TNT, or _Thom's Nuxt Template_, is a Nuxt theme/component library/set of conventions. It's a set of templates plus a collection of third party libraries that I frequently install so... I packaged them all up in the one Nuxt module for ease of installation. It's a work in progress, but it's already pretty strong. Any new static website I want to deploy, I essentially just need to generate a Nuxt project, install this one package and I am ready to go. Despite this, TNT has not yet hit its first _release_, by which I mean I've not yet published the package on NPM. It is currently installable only as a Git dependency, available here: https://github.com/thombruce/tnt. There's a fair bit that still needs ironing out, like the appearance of the classic template on mobile screens, ease of configuration of custom menus, titles and the like, but it does already make getting started with Nuxt roughly five times quicker! It's getting updated all the time as I add more to my static websites and produce others. Speaking of which...

## My Static Websites

I actually started TNT to collect my conventions as I worked on these. I currently have eight static websites deployed that use TNT. They are:

1. [thombruce.com](https://thombruce.com/)
2. [Code](https://code.thombruce.com/)
3. [Happy](https://happy.thombruce.com/)
4. [Popcorn](https://popcorn.thombruce.com/)
5. [Ink](https://ink.thombruce.com/)
6. [Ordahhh!](https://ordahhh.thombruce.com/)
7. [Free as in Beer](https://beer.thombruce.com/)
8. [TNT Docs](https://thombruce.github.io/tnt)

A lot of these are very simple blogging websites. Free as in Beer is intended to be a sort of catalog website of freely available software and resources, and TNT docs is a documentation website; these two need a bit of work yet, but TNT is working for the others beautifully.

### Journal, Know It All and Review

That's a lot already, but I still have other projects needing documentation and these three other blogging sites I'd like to throw onto the web. The idea is that... well... I have a lot that I want to say. Or I don't, but when I do I may want to talk about any of a bunch of subjects that I'm invested in: science, politics, film and TV, programming, writing and so on and so on... But readers or visitors to a website in general don't want... all of that. For example, if I had a science blog that took a left turn into creative writing or opinion, this would not be something necessarily that the readership wanted. This is why I don't just maintain all of these blogs at thombruce.com or blog.thombruce.com or whatever; as a general rule, I'm separating interests into individual domains. These three left to do represent science, opinion and personal blogging; _Know It All_, _Review_ and _Journal_ respectively. It won't take a lot to get them online; I just haven't done it yet. Each one that I do put online does require a slight change to my main site, [thombruce.com](https://thombruce.com/), which does list all blog posts across all sites, read from a JSON feed each of them exposes. Though, speaking of that...

### Connected Events in Static Deployment

This currently doesn't work automatically. When I deploy a blog post to one of my sites, I currently need to also manually redeploy thombruce.com. Preferably, each site build would automatically trigger the redeploy of thombruce.com. Alternatively, I could rewrite the code on thombruce.com to fetch the data at runtime - meaning when a visitor arrives on the site. This would be... slower, less efficient. It's less worth considering than the preferable approach.

To achieve the desired result, I really just need to trigger a GitHub Action, either via GitHub's API or via a webhook; both documented here: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#webhook-events

The issue is that the build server and deployment server for each of my sites are... different. There could be lag, and I could potentially trigger the thombruce.com rebuild too early. So I still need to figure out where to place this logic precisely and how I go about that.

## Hosted, Marmalade and Dynamite CMS

Everything mentioned so far depends almost exclusively on Nuxt, TNT and their dependencies. Nuxt is great, and I love what I've put together with TNT, but what's missing here is the backend. I'm using Nuxt as a static site generator, so it's building websites by reading Markdown documents and inserting those into HTML templates that go up to a server and can be served as-is to any visitor. But this severely limits things like interactivity, storage and broader functionality. Plus... I'm writing these blog posts in a code editor right now; the formatting is readable but it isn't pretty, and it doesn't even try to approximate what it will look like to a website visitor - it's plain text. In fact the ways in which I'm managing all of these websites, including their domains, DNS resolution, hosting and building... It's all... disconnected. There's more that I want to do there, and there's more that I want to do in general than Nuxt alone can achieve.

For example, _Hosted_ is a concept I'm planning for a web management dashboard that would integrate all of these systems and more into one dashboard from which I could see an overview of details for all of my websites. _Dynamite CMS_ would be a content management system that would pretty up these blog posts at the time of my writing them, offering me and other users a more familiar writing experience. And _Marmalade_.... _Marmalade_ is an old project I'm still very interested in revisiting; it would be a personal and business management dashboard, integrating much of what you might need to keep both aspects of your life, businesses and brands in order. All of these require backends that either store critical data, or communicate with third party services securely. And for that... for the past ten years... for that I've used Ruby.

### Returning to Ruby: Playing Nicely with Nuxt

So Ruby, or more specifically Ruby on Rails, is a web framework that I started using about a decade ago. In the past few years, I've been writing code for it in only a professional capacity as my more personal interests diverged into static site generation and Nuxt, but... I think it's the backend I do want to return to. I love Ruby. But I also love my new approach to the frontend. The question is, and it's a big unanswered one for me... can I get Nuxt and Ruby to play nicely.

Preferably, I would have a single repo per project that has both frontend and backend applications which can - but don't necessarily have to - run on the same server with a single click deploy. Getting them to work on different servers is trivial; but what about the same one? At deploy time, Ruby needs to run some asset precompilation, database migrations and then it runs a full application on a server, performing computations for every visitor. Nuxt, meanwhile, needs to build at deploy time and then it can be served either rendered on server by Node.js (I think) or as static JavaScript files injected into HTML documents. So there are some choices there and questions about what's even possible... All of it is, technically, possible. But if we're strictly aiming for a single-click deploy, for ease of other users installing this themselves, then we may run into issues. Can one of the hosting platforms offering such deploys handle my demands here? Can both of the ones I'm currently using (Heroku and Railway) do so? I don't know. It remains to be seen, but I'm itching to get started on these projects so I will visit that idea again real soon.

## Project "Dashboard"

Finally, much as with my blogging, not everything I want to do or want to show off really fits in any existing project, but I also don't want to necessarily start some new project either. I have this idea to host a sort of... dashboard of widgets showing off various web development knowledge I already have or will obtain in one place. A dumping ground, essentially, for random code like... here's a clock, here's a weather forecast, here's a map and here's a 3D model of the moon, or whatever. Maybe I'll call it "snippets" and try to make the code snippets as easily viewed as the actual results as well. We'll see. It's on the back burner for now, but I'm interested in throwing it together quickly using Nuxt and TNT; we'll see if it too needs a Ruby API for a backend... or, in fact, several, dependent on the type of data each concept needs.

## The Future: Refining My Computer Science Skills and Knowledge

Okay, and finally, finally... I know a whole lot about programming. I do. I've been doing this for almost ten years professionally, for practically twenty as a hobbyist and longer still if we count the little bit of programming we learned in primary school. But despite all that I do know about programming, I have forgotten a whole lot about mathematics. I was watching a Matt Parker video recently in which he calculated the distances of words as a product of the millimetre distances between keys on a keyboard. To do this, he used Pythagoras', trigonometry and a little bit of something called _the dot product_ which... I haven't even heard of. This is stuff I should know - I learned trig and Pythagoras in secondary school, and I remember a bit, like a^2 + b^2 = c^2 - that's Pythagoras - but can I remember how to apply it or make use of it? Not really.

There are gaps in my knowledge, of mathematics, of computer science, and I should aim to plug those gaps. So I... don't know how I'll go about that yet. I am thinking of looking for a refresher mathematics course and then diving into more materials from there. But eventually... this will enable me to program a lot, lot more. Physics simulations, artificial intelligence perhaps, crypto... I've done a little crypto, I understood only half of what I did though. So a lot to cover. A lot to do. Best get started...