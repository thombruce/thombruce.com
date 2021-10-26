---
title: "Reworking TNT: Creating a Vue Plugin - Part 2"
description: 'Working through Webpack and other bundler conflicts and dependencies'
authors:
  - Thom Bruce
date: 2021-08-30T17:42:29Z
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

[Last time](reworking-tnt-vue-plugin-part-1) we ran into a bit of a blockade trying to make my TNT plugin provide its own default configuration. The issue: Vue CLI, which we used to generate the Dynamite UI project I'm working on, obfuscates a lot of the configurability of Vue and Webpack. Under the hood, Vue CLI is using Webpack... _I think_. And this can be configured from the project with our own Webpack config files, I presume, but the obscurity is making it difficult to assess the relationship between the UI project and my plugin.

Today then, we're going to start from scratch and create a more barebones Vue project. This should be a minimum viable reproduction of what we ultimately want to achieve... meaning essentially we just really want two dependencies: Vue and TNT. We might find we need more, which may either need to be added to the Vue project directly or, preferably, to TNT, but the result should be a plugin that works with plain Vue projects, with Vue CLI ones and with Nuxt, because those latter two approaches build upon the first. In fact, this is what this whole "Reworking TNT" exercise has been about: prying TNT apart from the Nuxt logic that it previously depended on. Let's get rolling... First, we need a new project:

Step one: Create an empty project folder and initialise Git.

```sh
mkdir my-vue-project
cd my-vue-project
git init
```

There's nothing to commit. The project is still empty.

Step two: Initialise Yarn.

```sh
yarn init
```

This runs an interactive session. I'm not going to change any of the default values, though; this is just a throwaway project.

Step three: Install the two main dependencies - Vue and TNT - and link to our local version of TNT for development.

```sh
yarn add vue https://github.com/thombruce/tnt
yarn link @thombruce/tnt
```

One thing worth noticing is that at this point, a `node_modules` directory will have been generated. We don't want that to be committed, so we'll add a `.gitignore` file to the root of our project with `node_modules/` in there to be ignored by Git:

```sh
echo "node_modules/" > .gitignore
```

And let's `git commit`...

```sh
git add . && git commit -m "Initialize project"
```

So, Vue and TNT along with all of their child dependencies are now installed...

Next, I'm just going to copy over some files from my Dynamite UI project; specifically, `src/App.vue` and `src/main.js`. These are the main components that tell Vue what my app looks like and how to render it; they just need some minor alterations for this more basic setup. Removal of any references to VueRouter and the Vuex Store, neither of which is installed. After that, my `src/main.js` looks like this:

```js
import Vue from 'vue'
import App from './App.vue'

import TNT from '@thombruce/tnt'

Vue.config.productionTip = false

Vue.use(TNT)

new Vue({
  render: h => h(App)
}).$mount('#app')
```

I've also created a minimal `index.html` file in a new `public` folder, the main part of which is that it includes this:

```html
<div id="app"></div>
```

This is where the Vue project will ultimately be injected...

So I've got an `App.vue` which handles how the app will be rendered, I have a `main.js` which tells Vue how to render and it how to behave... and I have an `index.html` where it should be injected but... isn't, because I still do have to do _the next thing_.

Vue's own documentation is pretty shy on how to handle this part; the part where Vue is packaged into the HTML template. The docs really seem to favour Vue CLI, which handles all of this automatically. For manual configuration, they link us to Webpack's documentation. Webpack isn't our only option - one can argue that a package like Gulp is simpler and more elegant, particularly for simple cases, and there's Vite too which is a new kid on the package bundling scene from the creators of Vue. For right now though, I think probably Webpack is going to give us the easiest solution thanks to its having some preconceptions, in a manner of speaking. For instance, whereas Gulp would require us to write our own bundling script, Webpack abstracts that part and has some base assumptions about web projects. Like by default, Webpack will look for a `src/index.js` and will output the result of bundling that file to `dist/main.js`. We may need to install some custom loaders, like Vue Loader... maybe. But err... well, we'll get to that. So...

Step four: Install Webpack and other bundling necessities.

```sh
yarn add --dev webpack webpack-cli webpack-dev-server vue-loader vue-template-compiler style-loader css-loader postcss-loader html-webpack-plugin
```

That's err... a lot of dependencies. And there's at least one missing that we might have mentioned - pug-plain-loader - but this is already being installed by TNT, so there's no need to respecify it. All of these are loaders, bundlers and plugins for Webpack's packaging of the app as a webpage. Also notable: webpack-dev-server is a package that will let us run the bundled application in development mode, just makes it a little easier to check things are working.

Right, but this needs configuring... First, let's add the build and development build scripts to `package.json` for convenience:

```json
"scripts": {
  "build": "webpack --mode production",
  "start": "webpack-dev-server --mode development"
},
```

That way I can just run `yarn start` to run the dev server. But it still needs a Webpack configuration to reference... That's a bit more verbose.

In a new file, `webpack.config.js`, I have got:

```js
const { VueLoaderPlugin } = require("vue-loader")
const htmlWebpackPlugin = require("html-webpack-plugin")
const path = require("path")

module.exports = {
  entry: {
    main: "./src/main.js"
  },
  module: {
    rules: [
      {
        test: /\.vue$/,
        loader: "vue-loader"
      },
      {
        test: /\.pug$/,
        oneOf: [
          {
            resourceQuery: /^\?vue/,
            use: ['pug-plain-loader']
          },
          {
            use: ['raw-loader', 'pug-plain-loader']
          }
        ]
      },
      {
        test: /\.css$/,
        use: [
          "style-loader",
          "css-loader",
          {
            loader: "postcss-loader",
            options: {
              postcssOptions: {
                plugins: [
                  ["tailwindcss", {}],
                  ["autoprefixer", {}]
                ]
              }
            }
          }
        ]
      }
    ]
  },
  plugins: [
    new VueLoaderPlugin(),
    new htmlWebpackPlugin({
      template: path.resolve(__dirname, "public", "index.html")
    })
  ]
}
```

A pretty minimal Webpack configuration, all things considered. I don't know if it's the **absolute** minimum for my needs, but it's minimal enough.

Apart from this, I also have a `tailwind.config.js` file with contents:

```js
module.exports = {
  purge: {
    options: {
      safelist: [
        /data-theme$/,
      ]
    },
    content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  },
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {},
  },
  variants: {
    extend: {},
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('daisyui'),
  ],
}
```

These contents could in fact be inserted into the Webpack config file instead, in the empty object at `["tailwindcss", {}],`... I could in fact also separate out the PostCSS config from Webpack into a `postcss.config.js` file, which includes that declaration. In fact...

That could be the answer to my problem.

See, I already know that Tailwind's config file location can be customised by setting its path explicitly in the PostCSS config: https://tailwindcss.com/docs/configuration#using-a-different-file-name

I can also change the path to the default PostCSS config file by passing the option to the PostCSS loader: https://github.com/postcss/postcss-load-config/issues/55#issuecomment-266375307

The combination of these two options should enable me to specify that my PostCSS config and my Tailwind config live in my TNT plugin's folder... right? Obviously.

And because the config files are JavaScript, I should be able to have them (or some other part of the stack) communicate that **if** a config file does exist in the root project directory, **then** do use that instead. That way, an end user would only have to install the essential dependencies, then reference our PostCSS config file, and things would... work seamlessly.

Is that the best approach? Probably not...

---

### Quick Aside

I mentioned pug-plain-loader specifically up above. Is this strictly a dependency of TNT or is it one that should be moved into the inheriting projects? I sort of just install it out of habit every time I install Pug. Strictly though, it is a Webpack dependency... meaning that if an inheriting project used Vite or Gulp or some other process to package things, it wouldn't be essential. So then...

```sh
cd ../tnt
yarn remove pug-plain-loader
git add . && git commit -m "Remove pug-plain-loader"
git push
cd ../my-vue-project
yarn upgrade @thombruce/tnt --latest
yarn add --dev pug-plain-loader
```

I also need to `yarn upgrade @thombruce/tnt --latest` over on Dynamite UI and `yarn add --dev pug-plain-loader` so that that continues to work as well. I don't need to modify any Webpack business there, because it seems Vue CLI still picks up on the use of Pug automatically... however I could instead use the `vue add pug` option to install Pug via Vue CLI template.

In fact looking at the package for vue-cli-plugin-pug at https://github.com/jaeming/vue-cli-plugin-pug... There isn't a great deal going on there. Just a `package.json` file with pug, pug-plain-loader and raw-loader dependencies and an `index.js` which chains into the Webpack setup. So hold up just a second! Could I create a "vue-cli-plugin-tnt" that does the same? That, for example, includes all of the Webpack and loader dependencies I would want and chains into the Webpack process to configure those? Answer...

Yes, I absolutely could do that for Vue CLI projects. It would list TNT as a dependency, configure it automatically, and we'd be rolling...

So then... we should absolutely keep TNT Core, let's call it, as barebones as possible. Make no assumptions that the end user will be using Webpack or some other bundler, or Vue CLI or Nuxt, and instead maintain separate dependencies which specifically install it for Vue CLI and Nuxt, and... for plain Vue? We can aim to document the far more involved setup that includes a heap of extra dependencies... at least for the time being. In future, we can add a production-ready build, maybe. Will have to look into that.

The thing is... I don't want this to be a big, unproductive diversion. The main thing I want is for TNT to be compatible with the toolings I will be using specifically, meaning just Nuxt and Vue CLI for the time being.

So then...

---

We will have to consider better approaches for installation on a plain Vue project at some other time. For our purposes now, though, TNT has been stripped of all Nuxt-specific dependencies and of Vue CLI specific ones.

I'm already maintaining a nuxt-tnt package that does not yet depend on this alternate version of TNT, but will eventually.

The last thing to do as of right now is to create an additional project that bundles TNT as a Vue CLI dependency. In the next post, we'll do just that...