<template lang="pug">
div(class="w-[95vw] max-w-2xl mx-auto my-2")
  NuxtRouteAnnouncer/

  header(class="mb-4 space-y-2")
    a(href="/" class="block text-4xl font-pixel") Thom Bruce

    nav(class="space-x-2")
      NuxtLink(to="/") Home
      NuxtLink(to="/blog") Blog
      NuxtLink(to="/code") Code
      NuxtLink(to="/about") About
      NuxtLink(to="/contact") Contact
      ContentSearch(class="inline-block")/

    div
      ColorMode/

  main
    NuxtPage/

  footer(class="mt-4")
    div(class="flex flex-wrap space-y-4")
      section(class="flex-1 min-w-2xs")
        h2(class="text-2xl font-pixel") Extra
        nav
          ul
            li
              NuxtLink(to="/screenplays")
                span(class="icon-[fa7-solid--masks-theater] mr-2")/
                | Screenplays
            li
              NuxtLink(to="https://github.com/thombruce/.dotfiles")
                span(class="icon-[fa7-solid--screwdriver-wrench] mr-2")/
                | .dotfiles
            li
              NuxtLink(to="https://github.com/thombruce/cv/releases/latest/download/Thom-Bruce_CV.pdf")
                //- NOTE: Uses a little CSS magic to apply the appropriate icon
                //- TODO: I don't necessarily love the proper use of custom attributes though;
                //-       being "data-icon" instead of just "icon". Might revert to usage of a
                //-       class. Must settle on this.
                span(data-icon)/
                | CV
      section(class="flex-1 min-w-2xs")
        h2(class="text-2xl font-pixel") Socials
        nav
          ul
            li
              NuxtLink(to="https://github.com/thombruce")
                span(class="icon-[simple-icons--github] mr-2")/
                | GitHub
            li
              NuxtLink(to="https://bsky.app/profile/thombruce.com")
                span(class="icon-[simple-icons--bluesky] mr-2")/
                | Bluesky
            li
              NuxtLink(to="https://linkedin.com/in/thombruce")
                span(class="icon-[simple-icons--linkedin] mr-2")/
                | LinkedIn
    section(class="text-center")
      small Copyright Thom Bruce 1989 - 2025
</template>
