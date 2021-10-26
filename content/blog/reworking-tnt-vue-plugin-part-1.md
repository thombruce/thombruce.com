---
title: "Reworking TNT: Creating a Vue Plugin - Part 1"
description: 'Moving Vue Meta and Tailwind CSS into my TNT Vue plugin'
authors:
  - Thom Bruce
date: 2021-08-29T15:18:52Z
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

[When we left off](reworking-tnt-vue-meta) I had added the dependency for Vue Meta to TNT and had installed and configured it in Dynamite UI. This was a problem, as preferably TNT would install its own dependencies without any extra configuration being required. So that's what we'll do today, and in doing so we will make TNT a sort-of "proper" Vue plugin. Let's look at how Vue Meta is initialised as an example:

```js
import VueMeta from 'vue-meta'

Vue.use(VueMeta)
```

This is what we placed in Dynamite UI's `main.js`. It installs Vue Meta as a Vue plugin. And this is something we want TNT to do instead. Meaning that actually that exact code should live somewhere in the TNT project, and Dynamite UI should instead feature:

```js
import TNT from '@thombruce/tnt'

Vue.use(TNT)
```

...and to achieve that, TNT needs to export an `install()` function. This is all well documented in the Vue docs: https://vuejs.org/v2/guide/plugins.html

Anyway, my install function is going to live in a new file in my TNT project at `src/index.js`:

```sh
cd tnt
touch src/index.js
```

```js
import VueMeta from 'vue-meta'

export default {
  install (Vue, options) {
    Vue.use(VueMeta)
  }
}
```

When any project installing TNT includes `Vue.use(TNT)` now, it should run everything inside of the `install` funtion. In this case, just `Vue.use(VueMeta)` for now. Let's see if that's worked:

```sh
cd ../dynamite-ui
```

Then in `src/main.js`, I replace my old Vue Meta imports with TNT:

```js
import Vue from 'vue'
import TNT from '@thombruce/tnt'

Vue.use(TNT)
```

And see if it runs...

```sh
yarn serve
```

Ah-ha, problem... Unusual problem. Dynamite UI has ESLint installed and is complaining that it can't find an ESLint configuration for the files `../tnt/src/index.js`:

```text
Syntax Error: Error: No ESLint configuration found in /Users/thombruce/Developer/tnt/src
```

I expect this is a problem with `yarn link` linked packages and wouldn't occur for a more typical installation (indeed it hasn't cropped up for other dependencies). Fortunately, there's an easy solution. In my Dynamite UI project, I just need to add to `vue.config.js`:

```js
module.exports = {
  configureWebpack: {
    resolve: { symlinks: false }
  }
}
```

If I now try to `yarn serve` again...

Success! Vue Meta is successfully being installed as a sort of sub-plugin of TNT. Perfect. But the configuration is still being handled by Dynamite UI...

That's not strictly a problem. In the absence of the `data-theme` attribute configured on my main Vue component, Daisy UI simply falls back to default theme behaviour. This is, in fact, preferable to a default configuration of my own as I believe it adjusts based on light/dark theme settings on the host machine. Let's move onto TailwindCSS config...

---

There's quite a lot more going on to configure TailwindCSS in Dynamit UI. We have a `tailwind.config.js` file and we need to manually `import './index.css'` in `main.js`. But these items don't yet diverge from what I would like TNT to provide as default behaviour. Ergo, TNT should handle these configurations and imports if possible.

The second part is easy. I'm just going to recreate the CSS file in my TNT project at `src/assets/index.css`:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

...and import it from TNT's `src/index.js` file:

```js
import VueMeta from 'vue-meta'

import './assets/index.css'

export default {
  install (Vue, options) {
    Vue.use(VueMeta)
  }
}
```

Easy peasy! I can delete the references to this CSS file from the host project, Dynamite UI, and it's still working as intended. Perfect!

But the actual configuration of Tailwind is a little more difficult...

The question is: How does Tailwind know that it's installed? It's a better question than it sounds like. We've just discussed how TNT and Vue Meta are each acknowledged by the Vue app, but how does Vue know about Tailwind? Well...

As well as a `tailwind.config.js` file, my project also features a `postcss.config.js` file with the contents:

```js
module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
```

I believe we've discussed PostCSS previously - we had some version trouble. PostCSS is apparently part of the default Vue configuration, so Vue behind the scenes knows to look for this configuration file, and PostCSS in turn knows to look for these listed plugins. This, I believe, is all down to a hidden Webpack configuration. Webpack is a great tool, handles the packaging of... web things. Doesn't matter; point is it's doing this behind the scenes because of Vue CLI, which is the way that I generated this project. Vue CLI is cleverer than plain Vue - it brings along a lot of preconfigured conventions... a lot like Nuxt... so... a lot like what we've been trying to avoid. Thing is... my project should be installable in a Vue CLI project, in a Nuxt one, or just in a plain old Vue one. And I think understanding the configuration of a plain old Vue one would inform decisions made for Vue CLI and for Nuxt. We in fact already have a separate repo for Nuxt installations - might we also consider one for Vue CLI? They will, _I think_ be incompatible with one another, but that's all the more reason to have a base project that works a certain way with just plain old Vue.

...

All of that is to say, I have made a mistake. Dynamite UI was generated with Vue CLI, but I need to take a step backwards, go another layer up the abstraction ladder and consider how a very simple Vue project would install this package...

In other words... I'm going to call this entry right here and we'll begin a fresh project in the next one.