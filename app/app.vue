<template lang="pug">
div(class="mx-4 my-2")
  NuxtRouteAnnouncer/

  header(class="max-w-prose mb-4 space-y-2")
    span(class="block text-4xl font-pixel") Thom Bruce

    nav(class="space-x-2")
      NuxtLink(to="/") Home
      NuxtLink(to="/blog") Blog
      NuxtLink(to="/about") About
      NuxtLink(to="/contact") Contact

    div
      ColorMode/

  main(class="max-w-prose")
    NuxtPage/
</template>
