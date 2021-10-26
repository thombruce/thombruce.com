---
title: "Reworking TNT: Addendum - Bundling the Plugin"
description: 'Using Vue CLI to bundle my Vue plugin as a JavaScript module'
authors:
  - Thom Bruce
date: 2021-09-02T22:48:00Z
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

So, in [my last entry](reworking-tnt-vue-cli) in this series I said...

> This installs TNT, Tailwind, DaisyUI, Pug, Lodash, Luxon and Vue Meta. My next step is to flesh this thing out with those features I wanted to bring across from Nuxt in the first place. And that ends this short blog series.

...and I drew a line under this series of posts. I said _hey_, y'know, _I'm done here_. Developers should never be hubristic... no matter how trivial the hubris.

The problem:

```txt
Syntax Error
Cannot use import statement outside a module
```

To be fair, it worked at the time of my writing that last post. I've since made some changes to try and integrate the new Nuxt-free TNT plugin with the existing Nuxt plugin a little better. Specifically, the failing import is the import of the plugin itself. As of writing my last post, I was benefiting from the installation of dependencies from the one package, but I wasn't actually importing the main part of the plugin itself - I was still largely depending on the Nuxt code, some of which is just duplications of the Vue code now. Naturally, I wanted to trade that out and so... now it's broken. At least, the Nuxt TNT module is broken. The plugin still does work... just not inside of the Nuxt environment. Why? Well... That's complicated.

Y'see, there's more than one way to handle an `import` statement in JavaScript. There are essentially what I am going to call _different flavours_. And there are a few of them, but I want to address just two in particular: CommonJS and ES6 Modules. And I'm... not actually concerned about the differences between the two. One pre-parses and resolves sub-dependencies (ES6), the other loads and executes in order (CommonJS). This doesn't actually concern me. This may not actually be my problem, in fact. I just raise it because I... _could_ theoretically resolve the issue personally by installing one or the other, and configuring it a certain way in my own projects and _hey presto_, that should work. But that's working around the problem, avoiding the meat of it, and if others try to use my plugins and encounter the same thing, then I need to tell them to use this nasty workaround instead of my actually fixing the issue. It's just not the proper way to do things.

So... how should I actually solve this? Well, by actually bundling the plugins properly... or at least... I assume that will work.

See, an NPM module can be bundled into separate syntaxes, each a different version of the code with slight variations for different environments and saved to files with different extensions. For instance, Vue itself comes in a standard flavour at `vue.js` and an ES module flavour at `vue.esm.js` - I've ran into this little difference on more than one occasion. Essentially, we want to do that. But does that mean rewriting the code twice..? Of course not. The code as it stands is functional, and doesn't need to be changed. It just needs to be interpreted by a bundling tool, with different versions spit out for these different environments. And I think the way we're going to do that is...

...by using our new friend, Vue CLI. This is going to be the easiest approach initially, though not necessarily the best. Even Vue CLI's own docs recommend a different tool for bundling plugins, [Rollup](https://rollupjs.org/guide/en/). As I'm not familiar with that tool, I want to leave it a consideration for the future. It does apparently result in smaller bundle sizes, which is great, but this isn't my main concern right now. My main concern is bundling a module at all. When I do swap out for Rollup in the future, I believe it will be easier to do so if we already have the target behaviour implemented - that way we have an exact goal to aim for.

Unfortunately, Vue CLI isn't installed in my main TNT project just yet, so I'll have to add the dependencies manually:

```sh
yarn add core-js vue
yarn add --dev @vue/cli-plugin-babel @vue/cli-plugin-eslint @vue/cli-service babel-eslint eslint eslint-plugin-vue vue-template-compiler
```

I've also copied across the ESLint and Babel configs from a freshly created Vue CLI app. More important though are the changes I'm about to make to `package.json`...

I change the location pointed to by the `main` option and add a new `build` command to the `scripts` object:

```js
"main" : "dist/tnt.common.js",
"scripts": {
  "build" : "vue-cli-service build --target lib --name tnt ./src/index.js",
},
```

Then I can run:

```sh
yarn build
```

...and Vue CLI spits out a set of files including `tnt.umd.js` to a newly created `dist` folder.

And that's it! ... Or should be, at least. I need to now go away and test this out, see if I can `import` everything correctly.

_Ugh... Yup. Yup, I can. I just have a whole new problem now._

So, that worked but it looks like I now have some kind of conflict between Nuxt's use of Vue Meta and the version of Vue Meta that I invoke for Vue projects. Not a problem...

We'll just conditionally invoke Vue Meta based on whether or not the project is running Nuxt. I won't bother thinking about how to check for that implicitly just now; instead I'll just look for a truthy `nuxt` option in the options object:

```js
import VueMeta from 'vue-meta'

// ...

export default {
  install (Vue, options) {
    // ...
    if (!options.nuxt) {
      Vue.use(VueMeta)
    }
    // ...
  }
}
```

...and then when invoking TNT in a Nuxt project (in this case the Nuxt TNT package), we'll just set that option explicitly:

```js
import Vue from 'vue'
import TNT from '@thombruce/tnt'

Vue.use(TNT, { nuxt: true })
```

Nice and simple, not something end users will need to worry about. Something I might do to make this a little more general purpose and configurable in the future is to swap this boolean `nuxt` option out for an object of some sort that can pick and choose features from the plugin. But this is quick and easy, and good enough for now.

All right then... Worked myself into a little bit of a confused state here by working on a couple of projects at once, but I think I'm ready to commit. Some cleaning up to do, but other than that...

...commit and push my TNT changes, `yarn upgrade @thombruce/tnt --latest` from my Nuxt TNT project. And then gradually upgrade the packages wherever I use them.

With that done, the next step really is publishing these and making sensible use of semantic versioning. But that for now is a task I leave unwritten. End of chapter.