---
title: "Reworking TNT: Creating a Vue CLI Plugin"
description: 'Installing dependencies, loaders and configurations with a Vue CLI Plugin'
authors:
  - Thom Bruce
date: 2021-08-31T04:58:30Z
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

[In my previous post](reworking-tnt-vue-plugin-part-2) we tried to figure out how TNT could be installed with all those dependencies I love and use often (e.g. Pug, PostCSS, Tailwind) as part of a Webpack bundle while not strictly confining end-users to the use of Webpack. That is, it should be usable with different bundlers like Vite and Gulp as well. In doing so, we created a minimal Webpack installation and examined the parts that make my dependencies and the TNT package work. We were trying to avoid making it specific to Webpack, to avoid also making it depend upon Vue CLI, so that it could have broader use. And yet, we learned how a Vue CLI plugin communicates its intentions to bundle a Webpack package by examining the Pug Vue CLI plugin, and we discovered that this is simple, straightforward and perhaps elegant. In other words, we have taken the long way around trying to avoid creating a Vue CLI plugin in order to ultimately that that is precisely what we actually want to do. Still, it has been informative and the lessons learned can be used to document the installation of TNT without Vue CLI too.

In this post, I finally intend to finish this project by making a Vue CLI plugin for TNT that communicates its dependencies and the essential Webpack configuration that will make it work just a single line of code. By the end of this article, TNT should hopefully be as easy to install as...

```sh
vue add tnt
```

Let's get to it...

---

## Initialising the Plugin

So, we have a goal in sight - `vue add tnt` - and we have one example of how that can be achieved - https://github.com/jaeming/vue-cli-plugin-pug - now we just need to get there. We'll start by looking at the docs for creating a Vue CLI plugin like that one...

Vue have a plugin development guide for Vue CLI plugins here: https://cli.vuejs.org/dev-guide/plugin-dev.html I caught a glimpse of this the other day, and I'm particularly interested in something that caught my attention then: discoverability. Their section on [Naming and discoverability](https://cli.vuejs.org/dev-guide/plugin-dev.html#naming-and-discoverability) insists upon a naming strategy matching the form `vue-cli-plugin-<name>` or `@scope/vue-cli-plugin-<name>`. We know the `<name>` is going to be 'tnt', but what about the `@scope`? The typical scope I've used is my own name - e.g. `@thombruce/tnt`. I could continue to follow that pattern or make a change in this particular case... My question is: How does Vue CLI handle conflicting names? Answer... Found here: https://cli.vuejs.org/guide/plugins-and-presets.html#installing-plugins-in-an-existing-project Elsewhere in the docs, the guide makes clear the difference:

> You can also use 3rd party plugins under a specific scope. For example, if a plugin is named @foo/vue-cli-plugin-bar, you can add it with: `vue add @foo/bar`

So it's my choice, then. Do we want `vue add tnt` or `vue add @thombruce/tnt`? I'm gonna opt for scoped for the time being. Part of the reason that my other packages are scoped is... 'tnt' as the name of an NPM package was already taken by a package published 8 years ago and never updated. I'm happy with scoped, but I may change things up later. Let's create the package...

```sh
mkdir vue-cli-plugin-tnt
cd vue-cli-plugin-tnt
git init
yarn init
# name: @thombruce/vue-cli-plugin-tnt
# version: 0.1.0
# description: vue-cli plugin to add TNT with Pug, Tailwind and DaisyUI
# entry point: index.js
# repository url: https://github.com/thombruce/vue-cli-plugin-tnt
# author: Thom Bruce
# license: MIT
# private: false
touch index.js
touch README.md
touch .gitignore
```

We'll leave `index.js` empty for just the moment, but I'm adding a little title and description to `README.md`. I've also copied across the contents of a boilerplate `.gitignore` that's good for general purposes. And then...

```sh
git add . && git commit -m "Initial commit"
```

I've also created the repository on GitHub and will push my initial commit now...

```sh
git remote add origin https://github.com/thombruce/vue-cli-plugin-tnt.git
git push -u origin main
```

The project now exists at https://github.com/thombruce/vue-cli-plugin-tnt. At present it lists none of the required dependencies, and it provides no instructions for initialising TNT. So that's our next step.

## Install Dependencies

Let's go over the dependencies we installed last time...

- webpack
- webpack-cli
- webpack-dev-server
- vue-loader
- vue-template-compiler
- style-loader
- css-loader
- postcss-loader
- html-webpack-plugin
- pug-plain-loader

Interesting list. All of these are in fact already a given, with the exception of `pug-plain-loader`. Webpack will already be installed, as will Vue and the style and CSS loaders, even the PostCSS loader is one we can take for granted.

Meanwhile, TNT itself lists the dependencies for Tailwind, DaisyUI, PostCSS, Pug, Vue Meta and Autoprefixer. We just need to include TNT as a dependency here, and provide the logic to be hooked into Webpack. So, it's just the two dependencies? Hmm... Three. We should add `raw-loader`; the Pug Vue CLI includes it, and my own experimental project made use of it without ever declaring the dependency explicitly (it might've failed if I'd had `.pug` files instead of Vue Single File Components). So...

```sh
yarn add https://github.com/thombruce/tnt pug-plain-loader raw-loader
```

_NOTE: TNT is not yet published on NPM. Shouldn't be a problem, but we'll revisit these dependencies in the future when it should be referencable by name instead of GitHub repository URL._

## Integration Logic

Last thing to do then (before we work out how to publish this as discoverable by Vue CLI) is to write the Webpack logic that should chain my steps and configuration into that used by default in Vue CLI projects.

In order to install Pug, we will be copying across the contents pretty much verbatim from https://github.com/jaeming/vue-cli-plugin-pug/blob/master/index.js. That's the Vue CLI Pug plugin, and what the logic in that file does is essentially just tell Webpack how to handle `.pug` files when it finds them and the contents of Vue Single File Components that have `<template lang="pug">` as part of their contents. We want TNT to do the same, to do **exactly** the same, so we're copying that.

In our plugin's `index.js` file:

```js
module.exports = (api, options) => {
  api.chainWebpack(webpackConfig => {
    // Remove any existing rule added from a previous version of the plugin (npm uninstall/ yarn remove will remove the plugin, but leave behind the webpack rules)
    webpackConfig.module.rules.delete('pug')

    // Rules taken from: https://vue-loader.vuejs.org/guide/pre-processors.html#pug
    webpackConfig.module
      .rule('pug')
        .test(/\.pug$/)

        // this applies to <template lang="pug"> in Vue components
        .oneOf('vue-loader')
          .resourceQuery(/^\?vue/)
          .use('pug-plain')
            .loader(require.resolve('pug-plain-loader'))
            .end()
        .end()

        // this applies to pug imports inside JavaScript, i.e. .pug files
        .oneOf('raw-pug-files')
          .use('pug-raw')
            .loader(require.resolve('raw-loader'))
            .end()
          .use('pug-plain')
            .loader(require.resolve('pug-plain-loader'))
            .end()
        .end()
  })
}
```

But we do need to add our own adaptations in order to inform the existing PostCSS installation that it should use the tailwindcss and autoprefixer plugins we're including as dependencies. And as well as this, we want to provide some default configuration to the tailwindcss plugin.

To achieve this, I'm going to look at the existing Vue CLI Tailwind plugin by Jens Eggerstedt ([forsartis on GitHub](https://github.com/forsartis)), [forsartis/vue-cli-plugin-tailwind](https://github.com/forsartis/vue-cli-plugin-tailwind).

Eggerstedt's plugin includes some prompts which ask the installer if they want to create a full, minimal or no config at all for TailwindCSS. I'll be modifying this so that no prompts are asked, and my own specific TailwindCSS config gets created no matter what. Why? Well... it's going to be simplest, and for my own purposes I will generally be installing TNT any time I initialise a Vue project, so I want all of my default configuration when I do that - I want _conventions_.

So, now we're going to write a `generator.js` file for our plugin that installs the @thombruce/tnt dependency, injects the import and Vue.use for @thombruce/tnt into the project's `main.js`, and creates some default configuration files. The result of my efforts is this:

```js
module.exports = (api, options) => {
  const postcss = readPostcssConfig(api);
  const configs = {
    dependencies: {
      '@thombruce/tnt': 'https://github.com/thombruce/tnt'
    },
    postcss: {
      plugins: {
        tailwindcss: {},
        autoprefixer: {},
      },
    },
  };

  configs.postcss.plugins = { ...configs.postcss.plugins, ...postcss.plugins };

  api.extendPackage(configs);

  api.onCreateComplete(() => {
    importTNT(api);
    generateConfig(api, options.initConfig);
    injectPurgeConfig(api);
    injectTailwindPlugins(api);
  });
};
```

Now, the file itself does have a bit more in it than that. There are several functions here, but I just want to refer to them by name for the time being. What's going on here? Well...

1. The @thombruce/tnt dependency is being added.
2. The PostCSS plugin configuration is being altered.
3. These both happen in the `api.extendPackage(configs);` step.
4. With that done and "onCreateComplete", we then...
5. Generate the `import` and `Vue.use(TNT)` statements for the `main.js` file.
6. Generate a fresh TailwindCSS config with `generateConfig`.
7. Inject our own PurgeCSS configuration into that config.
8. Inject our own plugins configuration into that config.

And that's it. In fact, it works! I've tested the installation like so...

```sh
yarn add --dev file:/Users/thombruce/Developer/thombruce/vue-cli-plugin-tnt
vue invoke @thombruce/tnt
```

Pug is working, Tailwind is working, DaisyUI is working. Seemingly... all of it is working. And importantly, we have separated out the parts that make TNT work with Vue CLI and Webpack. The main TNT package now is... actually, it's pretty bare, but it has only the operational dependencies, rather than any platform-specific ones. So it's... stack-agnostic. Fantastic!

```sh
git add . && git commit -m "Install and configure dependencies"
git push
# Done!
```

It isn't yet published properly, but can be installed straight from GitHub and invoked in-project with...

```sh
yarn add --dev https://github.com/thombruce/vue-cli-plugin-tnt
vue invoke @thombruce/tnt
```

This installs TNT, Tailwind, DaisyUI, Pug, Lodash, Luxon and Vue Meta. My next step is to flesh this thing out with those features I wanted to bring across from Nuxt in the first place. And that ends this short blog series.