---
title: "Reworking TNT: Vue Meta"
description: 'Adding Vue Meta to my TNT Vue.js plugin'
authors:
  - Thom Bruce
date: 2021-08-28T00:59:19Z
categories:
  - Journal
series:
  - My Process
  - Reworking TNT
tags:
  - Vue.js
  - NuxtJS
  - TNT
  - Dynamite
---

In [my last entry](reworking-tnt-without-nuxt), I added a little data attribute directly to the HTML tag on my new project, Dynamite UI. This is something that I said I would ideally address in the future - a configuration that, actually, should be handled by the TNT plugin. Well... I'm back and I have the solution. It occurred to me while doing something else entirely, actually: there was nothing special about the way it worked with the Nuxt module, it was just [Vue Meta](https://vue-meta.nuxtjs.org/). I had thought it was something I'd added to my Nuxt config, which it was, but it was a configuration option that gets passed along to Vue Meta. So, what is Vue Meta?

Vue Meta is actually developed by the people behind Nuxt... at least, I think it is. It's official site is a subdomain of nuxtjs.org. It's a plugin that enables the addition of tags and meta attributes to the HTML document's `<head>` tag. This is typically off limits to Vue components, but Vue Meta makes it possible to define and overwrite attributes directly from Vue components. And for my purposes right now, it also exposes a `htmlAttrs` object for defining attributes for the `<html>` tag itself. That's all I need to avoid having to directly write my DaisyUI theme data attribute into the HTML template in all of my Vue projects - I can move it now into a configuration option. Let's install...

```sh
cd tnt
yarn add vue-meta
git add . && git commit -m "Install vue-meta"
git push
```

Unfortunately, I'm still not ready to configure features in the TNT Vue plugin, so I'll need to do that over on Dynamite UI...

```sh
cd ../dynamite-ui
yarn upgrade @thombruce/tnt --latest
```

I need to tell Vue to import and use it in my `src/main.js` file:

```js
import Vue from 'vue'
// ...
import VueMeta from 'vue-meta'

Vue.use(VueMeta)
```

When TNT is a proper Vue plugin, it will actually export a Vue install method and we will instead write something like `Vue.use(TNT)`. This then would include the code above, so that telling Vue to "use" TNT, it follows that TNT tells Vue to also "use" Vue Meta. We'll try to document that when we revisit and do convert this into a proper plugin. For now, Vue Meta is packaged by TNT but installed manually in the projects using TNT.

At this point, though, I should be able to... Remove `data-theme="cupcake"` from the `<html>` tag in `public/index.html` and...

Instead, add the attribute to... I guess `src/App.vue` for now:

```html
<script>
export default {
  metaInfo: {
    htmlAttrs: {
      'data-theme': 'cupcake'
    }
  }
}
</script>
```

And when I now `yarn serve`... Success! The DaisyUI 'Cupcake' theme is still being used, now courtesy of Vue Meta. Even better, this little exercise has made it a clearer how I go about turning TNT into a proper Vue plugin. That still to come, next time...