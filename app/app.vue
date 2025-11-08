<template lang="pug">
div(class="w-[95vw] max-w-2xl mx-auto my-2")
  NuxtRouteAnnouncer/

  header(class="mb-4 space-y-2")
    span(class="block text-4xl font-pixel") Thom Bruce

    nav(class="space-x-2")
      NuxtLink(to="/") Home
      NuxtLink(to="/blog") Blog
      NuxtLink(to="/about") About
      NuxtLink(to="/contact") Contact

    div
      ColorMode/

  main
    NuxtPage/
</template>
