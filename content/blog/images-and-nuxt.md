---
title: Images and Nuxt
description: 'Adding image support to my Nuxt blog.'
authors:
  - Thom Bruce
date: 2021-04-25T12:27:27Z
categories:
  - Jamstack
series:
  - Getting Started
tags:
  - Blogging
  - Images
  - Markdown
image: /code.jpg
---

I don't believe that any decent blogging software would be complete without support for images. Markdown has this baked-in with syntax that looks a little like this: `![](/path-to-image.jpg)`. Nuxt too could immediately support this syntax if I simply placed images into its `/static` directory, but... this approach wouldn't support image processing by default. The images would be loaded at their native size, regardless of the context they're being displayed in - whether that be on a mobile phone or a large desktop screen. The result would be some unnecessarily large files being loaded for mobile, and little in the way of control of the image content elsewhere.

One alternative is to use the HTML `<picture>` tag, which has support for loading different sources for different screens. This gives us significantly more control, but would result in some pretty cumbersome tags in our markdown content. Plus, images would need to be processed manually and we might end up storing numerous versions of our image at different sizes. I would prefer to save just one image, from which different sizes could be generated automatically.

Fortunately, there's the relatively new plugin, [Nuxt Image](https://image.nuxtjs.org/). Nuxt Image adds image processing to Nuxt, so that I don't have to fret too much about manually handling those things.

I have to do a couple of things to add it to my project. First:

```sh
yarn add --dev @nuxt/image
```

This adds the Nuxt Image plugin to my Nuxt website. But I also need to add it to my build modules, so that Nuxt acknowledges it as something I want to use when building my site from the source code. In `nuxt.config.js`, I add:

```js
buildModules: [
  '@nuxt/image',
]
```

With that done, I can now use Nuxt's `NuxtImg` and `NuxtPicture` tags wherever I like, including in my Markdown content. And with that, I can control the sizes displayed across different devices. Take note of the `sizes` attributes used below. Here, I've set it to the same value for sm, md and lg (small, medium and large screens), but these can be set individually so that different screens load different sizes. `50vw` and `25vw` below mean "50% of the view width" and "25% of the view width" respectively, so Nuxt actually will create differently sized images for differently sized screens, even though the values are the same. `vw` could have been `px` instead, which would have meant these were explicit pixel values, rather than sizes relative to screen size.

What all of this means is that I can now use the `nuxt-img` tag to generate images at different sizes; not merely the same image displayed at different sizes, but separate images specifically generated for the size that they will appear.

_Note: It is sort of poor practice to load the same image twice; if I want it twice, why not just load it once at the larger of the two values and artificially shrink the second version? The two examples below are included only for demonstration purposes._

### An image at 50vw

```md
<nuxt-img src='/code.jpg' sizes='sm:50vw md:50vw lg:50vw'></nuxt-img>
```

How it looks:

<nuxt-img src='/code.jpg' sizes='sm:50vw md:50vw lg:50vw'></nuxt-img>

### The same image at 25vw

```md
<nuxt-img src='/code.jpg' sizes='sm:25vw md:25vw lg:25vw'></nuxt-img>
```

How it looks:

<nuxt-img src='/code.jpg' sizes='sm:25vw md:25vw lg:25vw'></nuxt-img>

As stated in the note above, it's a little redundant to load the same image twice here at two separate sizes. An alternative to loading the second image would be to do the following:

```md
<nuxt-img src='/code.jpg' sizes='sm:50vw md:50vw lg:50vw' style='width:25vw;'></nuxt-img>
```

Instead of loading a separate, smaller version of the image, this would load the same version as the initial tag. The size as displayed is then controlled by CSS in the `style` attribute instead.

The result looks much the same:

<nuxt-img src='/code.jpg' sizes='sm:50vw md:50vw lg:50vw' style='width:25vw;'></nuxt-img>

The aim of course is always to only load precisely what is needed, and no more.

As a next step, I would really love to tap into the Markdown parser and override its image handling so that I could use `![](/path-to-image.jpg)` syntax and achieve the same result. I leave this, for now, as an exercise for the future.

_Image attribution: Photo by <a href="https://unsplash.com/@rirri01?utm_source=unsplash&utm_medium=referral&utm_content=creditCopyText">Rirri</a> on <a href="https://unsplash.com/s/photos/polaroids?utm_source=unsplash&utm_medium=referral&utm_content=creditCopyText">Unsplash</a>_