---
title: Anatomy of a Blog Post
description: 'In which I consider the attributes required to sort and associate posts in my blog content files.'
authors:
  - Thom Bruce
date: 2021-04-19T10:15:19Z
categories:
  - Jamstack
series:
  - Getting Started
tags:
  - Blogging
  - Nuxt.js
  - Markdown
---

In my last entry, I talked about my decision to use Markdown to write my blog posts and briefly touched on my use of a Static Site Generator, [Nuxt.js](https://nuxtjs.org/). Nuxt is extraordinarily agnostic about what those Markdown files should look like, however. In fact, it supports numerous file formats as well as Markdown and doesn't really express much of a preference. It does, however, come with an example Markdown file with a structure a little like this:

```md[file-name.md]
---
title: Title
description: 'A description of the content of the article.
This description can be multiline.'
---

The content of the Markdown article.

Another paragraph.
```

We might say this file has three attributes: the title, the description, and the content. In actuality, it has a few more; it's filename is another one (e.g. 'my-markdown-file.md'). For our immediate purposes though, let's just consider what we can see, those three in the example above.

So, all of these have some pretty clear utility, right? A title might be something that grabs the reader's attention or something we can sort alphabetically about, the description gives a further taste of what content to expect and... the content is the meat of the thing, that's what I want to write and ultimately to be read. But how do we order content by date? How might we show that another piece of content relates to this one?

Okay, so we're maybe missing a date attribute and a set of tags or a category attribute - actually, I like to add both. Here are a few attributes I like to add nice and early:

```md
---
title:
description:
createdAt:
updatedAt:
categories: []
series: []
tags: []
---

Content.
```

_Where a `[]` appears above, this denotes that the attribute may have multiple values._

So, there's the title, description and content as seen in the default example. I've added createdAt and updatedAt, each a date field that we'll use to show when a piece of content was added and whether it has received an update; we can also use these to sort our articles by time. Then there are the categories, series and tags attributes. These three are collections of attributes; in practice, series in particular may only list one, but I like to leave it open to having multiple values. We call these taxonomies, and they list values that will enable us to show the association between two pieces of content. For example, if two separate pieces of content are in the 'Programming' category, they can be listed together on a category page, or they can be recommended as further reading in the footer of either article. Tags are a more broad set of keywords, and series might be used to associate articles in, well, a series - _this one follows this one_.

We don't have to stop there, and I will certainly revisit this in future as I consider whether review article types require a score parameter and other such considerations, but we will leave it there for now. One big missed consideration just now is an image. Even if my blog doesn't display images, it remains useful to have an image associated with an article so that it will be displayed when we share it on social media; that type of image is called an OpenGraph image, because it is written into the OpenGraph data which Facebook and other services use to understand the content being shared. I'll revisit that in a future article too, I think. But just for now, having the ability to at least sort my articles by creation time is enough.
