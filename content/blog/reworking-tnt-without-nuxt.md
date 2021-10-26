---
title: "Reworking TNT: Thom's Nuxt Template without Nuxt"
description: 'Reconfiguring my Nuxt template module so that it also works with Vue.js'
authors:
  - Thom Bruce
date: 2021-08-26T22:36:35Z
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

If you don't already know, TNT is a work in progress Nuxt module I've been working on to simplify the setup of my Nuxt projects. It comes with TailwindCSS and DaisyUI, each with some pre-configurations so that things like theme-switching and CSS purging work. It also comes with the Nuxt Font Awesome module, my own Nuxt Taxonomies project, Lodash, Luxon and Pug... and it insists that Nuxt Content should be a peer dependency (meaning it doesn't install it itself, but your project would be expected to have it installed separately). So a lot of that is Nuxt-specific, but I would like to use Pug, Luxon, Lodash and Tailwind as common dependencies of my Vue projects too. And I would like many of the components I've written or will write for TNT to be available to Vue.js projects as well. What does this mean? Well... I'm going to have to do some rewriting. Ideally, I don't want to be maintaining two wholly distinct libraries to roughly do the same thing for two frameworks, particularly while the frameworks are very closely related. Instead... I either want one library that can be installed in projects of either framework, or I want my Nuxt-specific library to inherit from the Vue one, given that Nuxt is really just an extension of Vue. That's easier said than done. Like I said, this little library is heavily dependent on Nuxt at the moment. And it's possible we'll lose some of the desirable features of those Nuxt versions of libraries by replacing them with either packages made for just Vue instead, or general-purpose ones intended for any NPM-managed project. We'll have to ask the questions: What am I losing by replacing Nuxt Tailwind with just TailwindCSS? What am I losing by replacing Nuxt Font Awesome with Vue Font Awesome or with just Font Awesome? How do we reconcile the inclusion of Nuxt Taxonomies in Vue projects it's completely irrelevant to? If those losses are undesirable or even impermissible, what is the best approach to separate out Nuxt dependencies whilst keeping the core part of TNT's development in one place?

## Creating Dynamite

So, one small part of the reason I started TNT is that I thought such a template could ultimately be useful for creating websites for clients. It currently features two distinct styles of layout and, thanks to DaisyUI, more than a dozen themes. It's intended to be versatile, then. It's also intended to permit me to stop overthinking decisions about UI frameworks and CSS libraries. TailwindCSS makes every component very customisable, while DaisyUI provides a solid foundation for making those customisations. TNT, I thought, would be a strong starting point for any of my projects, whether it be a simple blogging website or a rich application dashboard. And honestly it's well on its way. But it's Nuxt dependent, and not all of my Vue projects will be Nuxt ones. At least I never intended for that to be the case, and now that I've reviewed how I might integrate a Vue or Nuxt frontend with a Rails backend... I don't think it can be. So, I need a new Vue project to play around with. For that, I'm going to make a start on Dynamite CMS, a sort of cousing to my TNT project - it's a content management system intended to work in particular with content websites created using TNT. We'll see if that intention remains the same as the project progresses, for now I just need a simple Vue application where I can play with my TNT template library. Let's create that...

```sh
vue create dynamite-ui
cd dynamite-ui
```

The `vue` command depends on Vue CLI being installed and runs an interactive menu allowing me to choose various options. Most of these aren't relevant to the discussion, except that I am specifically _not_ initialising the project with a CSS pre-processor. The project will be using TailwindCSS, PostCSS and - I think - PurgeCSS, which can't be installed that way. Plus, they will be part of TNT, not the parent project.

Here's what I'm thinking comes next...

```sh
yarn add --dev https://github.com/thombruce/tnt
```

This will install TNT, which I know I've said will be incompatible with Vue... but I don't really know just _how incompatible_ it will be. Maybe this way I can get a sense of what works, what doesn't, and what I need to change.

Predictably after running that command, there are a lot of things installed that I do not want. So I'm gonna jump on over to TNT...

```sh
cd ../tnt
git co -b vue
git push --set-upstream origin vue
```

I've also checked out a brand new branch called 'vue' and set the upstream branch on GitHub to this new branch. Not exactly what that means, but if you're unfamiliar with Git or version control at all... essentially now I can make as many modifications as I want without losing the previous state, which is saved as a separate branch on GitHub. Used properly, you always maintain a detailed history with version control meaning you can reset the project at any time to any previous point in time. It also facilitates collaboration, testing, etc. If you're a developer not using version control, start.

What next? Now I need to do what I just said and butcher TNT a little. Remove all the Nuxt dependencies and replace them with Vue or non-framework-specific ones.

The main script aside, the TNT project essentially consists of four folders: assets, components, layouts and plugins. Of these, assets and plugins should have no problems. We can incorporate their contents into a Vue project with relative ease. In my components and layouts folders, however... well, let's ignore layouts for now - it actually is Nuxt specific. The components folder, however, has several instances of `NuxtLink` and the `<Nuxt />` component being used. We can ignore the instances of the `<Nuxt />` component, as these are layout-specific, but those NuxtLinks do need to be addressed before the same components will work in Vue. That part's easy; for the most part, `NuxtLink` just becomes `RouterLink`... _I think that still works in Nuxt..._ But actually there are a lot of other problems with these components: uses of the `fetch()` hook, uses of Nuxt Content, Nuxt Taxonomies and some Nuxt variables. I don't want to sacrifice some of that, so I've made a decision...

There will at least be two directories (probably two separate libraries). One for Vue components, and one for Nuxt components. The Nuxt components will use, inherit from and extend the Vue ones with Nuxt functionality. The Vue ones will be more general purpose, unable to utilise some of Nuxt's features.

So we probably want to start a new project entirely. TNT should be general purpose, working for both Vue and Nuxt; a separate project, _Nuxt TNT_, should expand upon it. _Decision made!_

I've moved `@thombruce/tnt` to `@thombruce/nuxt-tnt` and have started a brand new project in the old namespace. The docs remain in the `@thombruce/tnt` repo, but other than that it is barebones at the moment. Time to start migrating some components and picking out our dependencies. I won't worry for now about having Nuxt TNT inherit from the new project - that can be left as an exercise for later. For now, I'm simply aiming to get some of that TNT goodness to work for Vue projects not using Nuxt.

After some messing around, I have a clean TNT repository with no code. But it's pushed to GitHub, so jumping back across to my new Dynamite UI project, I can install this empty package...

```sh
cd ../dynamite-ui
yarn remove @thombruce/tnt
yarn cache clean # To clean up any references to the old version and its Git history
yarn add https://github.com/thombruce/tnt
```

I've removed the old and installed the new. Now I want to be able to actively develop this and see the results on my own machine without having to push the dependency. Fortunately, there's `yarn link`.

```sh
cd ../tnt
yarn link
cd ../dynamite-ui
yarn link @thombruce/tnt
```

Now, Dynamite UI will use the local, linked version of TNT in development on my machine, so any changes I make will be applied almost immediately (definitely immediately if I have Hot Module Replacement up and running). Time to start actually adding dependencies and components to this thing!

---

The first dependency I want to install is Pug. I don't know how easy this is going to be though. Usually I can just `vue add pug` and depend on Vue CLI's implementation, but as I'm working on a plugin I think I'll take my old approach...

```sh
cd ../tnt
yarn add pug pug-plain-loader
```

Now, that's - I think - all that's required to get Pug working in the TNT plugin. Unfortunately I don't know if the new dependency will be installed over on Dynamite UI as a linked module... I may need to push TNT and `yarn upgrade @thombruce/tnt --latest`. `yarn link` was maybe not made for managing nested dependencies - it wasn't. But if I do remember rightly, Vue should have no problem with just the addition of pug and pug-plain-loader to dependencies. I think Webpack detects their presence, and so no additional config will be needed... at least once I do get this properly installed over on Dynamite UI. But since I need to push and then upgrade, I may as well get a few other dependencies installed too. I certainly want...

- Pug
- TailwindCSS
- TailwindCSS Forms
- DaisyUI
- Theme Change
- PostCSS (though I think this is already installed)
- Lodash
- Luxon

That's all of the dependencies from Nuxt TNT except the icon libraries and those that are Nuxt-specific. _We'll revisit icon libraries later._ Fortunately the only dependency listed above that I was using a Nuxt-specific version of is TailwindCSS; all others are general purpose. So picking the packages to install is pretty easy...

The TailwindCSS docs suggest an install command of `npm install -D tailwindcss@latest postcss@latest autoprefixer@latest`, which includes PostCSS (we believe already installed) and Autoprefixer, which we definitely want too, so... they join the installation:

```sh
yarn add tailwindcss@latest postcss@latest autoprefixer@latest @tailwindcss/forms daisyui lodash luxon theme-change
```

And since we need to push the project to upgrade it over on Dynamite UI...

```sh
git add .
git commit -m "Install dependencies"
git push
cd ../dynamite-ui
yarn upgrade @thombruce/tnt --latest
```

Checking the Yarn lockfile over on Dynamite UI, I can confirm that my dependencies are properly installed. To be doubly sure that at least Pug is working, I've modified the default app template of Dynamite UI to express itself in Pug format and I'm running `yarn serve`... Navigating to localhost:8080 in my browser... Yup, that appears to be working!

I'm also confident that Lodash, Luxon and Theme Change are all installed fine, as I will be importing these directly for the most part and don't need global access to them. They're installed, so they're installed successfully.

TailwindCSS, PostCSS, Autoprefixer, TailwindCSS Forms and DaisyUI... Those need more configuration, and that's not easy or immediately apparent how to achieve. With Nuxt, I benefited from a bunch of build hooks that I could tap into to: 1. Configure the installation in the TNT package itself, and 2. Allow for this configuration to be overwritten per project.

It's been a little while since I made a proper Vue plugin, so I don't know what options are available to me. For the time being, I think I'm going to configure TailwindCSS directly in the Dynamite UI project... That way I can get a feel for what sort of configuration needs to made and what I need to be able to overwrite.

```sh
# Still in dynamite-ui
npx tailwindcss init -p
git add . && git commit -m "Tailwind: npx tailwindcss init -p"
# Modify Tailwind config to enable PurgeCSS
git add . && git commit -m "Tailwind: Configure PurgeCSS"
# Configure Tailwind CSS
git add . && git commit -m "Tailwind: Configure default CSS"
```

These are the steps detailed at https://tailwindcss.com/docs/guides/vue-3-vite. I've also committed each step so that I will be able to review my git history and see these changes specifically, and in order.

TailwindCSS Forms is also very easy to add. Just require it from the plugins section in `tailwind.config.js` and...

```js
module.exports = {
  // ...
  plugins: [
    require('@tailwindcss/forms'),
  ],
  // ...
}
```

Done. Don't forget to commit the change:

```sh
# Add TailwindCSS forms as a Tailwind plugin
git add . && git commit -m "Tailwind: Add Tailwind Forms plugin"
```

DaisyUI and and Theme Change are a little more involved. In fact... I never fully got Theme Change working properly with the Nuxt package, though it was close enough - it changed themes, but it didn't successfully load the correct theme on subsequent visits. That's an issue I'll address in the future.

DaisyUI itself is easy enough to add. Like TailwindCSS Forms, all we need to do is require it as a plugin in our Tailwind config. We'll also safelist the data-theme attribute here so that DaisyUI's themes aren't purged by PurgeCSS:

```js
module.exports = {
  purge: {
    options: {
      safelist: [
        /data-theme$/,
      ]
    },
    // ...
  },
  // ...
  plugins: [
    require('@tailwindcss/forms'),
    require('daisyui'),
  ],
  // ...
}
```

Generally, I think it's supposed to be a good idea to require DaisyUI last... The DaisyUI docs say nothing about TailwindCSS Forms, but they do suggest that if we were using TailwindCSS Typography (a plugin for applying typographic styles to content over which we have little markup control) then it should be required after that because it extends some of its styles. We'll just roll with the assumption that DaisyUI should come last unless otherwise specified.

We also need to add that `data-theme` attribute to our HTML tag in `public/index.html`. There has to be a better way to do this, but for now we'll do this the easy way:

```html
<!DOCTYPE html>
<html lang="" data-theme="cupcake">
```

Let's commit all of that...

```sh
git add . && git commit -m "Tailwind: Add DaisyUI plugin"
```

Now, the issue is Theme Change... I'm actually going to double-back and remove Theme Change for now:

```sh
cd ../tnt
yarn remove theme-change
git add . && git commit -m "Uninstall theme-change"
```

Why do that? Because actually adding `data-theme` directly to the `<html>` tag above is a mess-enough for the time being, and because being able to change the theme on the fly isn't critical to my development. There are things I want to manage before we reach that point.

But where are we at now?

- TailwindCSS and DaisyUI should each be properly installed
- Pug we already know is working
- Lodash and Luxon are safely assumed to be ready to use

This puts me in position to start adding my own templates and components! For the time being, we'll grant that many of the steps above are required to get the TNT plugin working - we'll refine that in the future. The last thing to do at this stage is to create a layout and a component using Tailwind and DaisyUI; and if that works, we've completed this setup.

One quick copying over of one of my TNT layouts and... Error!

```text
Error: PostCSS plugin tailwindcss requires PostCSS 8.
```

Not to worry, I've seen this before. In fact, it's an issue that the Tailwind team are aware of an provide a solution to: https://tailwindcss.com/docs/installation#post-css-7-compatibility-build So then...

```sh
yarn remove tailwindcss postcss autoprefixer
yarn add tailwindcss@npm:@tailwindcss/postcss7-compat postcss@^7 autoprefixer@^9
git add . && git commit -m "Use TailwindCSS Compatibility version"
git push
```

This swaps out the latest version of Tailwind for a compatibility version, the same in every way apparently apart from that it maintains compatibility with older versions of PostCSS.

Let's jump back across to DynamiteUI...

```sh
cd ../dynamite-ui
yarn upgrade @thombruce/tnt --latest
yarn serve
```

Success! TailwindCSS is now working, DaisyUI and the chosen theme as well, my custom layout...

Now, it isn't ideal. A lot has been done on DynamiteUI that I'd rather be done in the TNT plugin, but this is a start. It's a lot easier to migrate that functionality up the stream so to speak into TNT than it is to fiddle endlessly with plugin functionality.

This is the first step. I'm in a position to start developing, because the dependencies I need are all now installed by the one package... they just aren't yet configured by it. We might cover configuring Vue plugins in another post soon, but I'm calling this one here. I have some tidying up and some developing to do.