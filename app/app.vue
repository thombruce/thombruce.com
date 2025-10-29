<template lang="pug">
div(class="mx-4 my-2")
  NuxtRouteAnnouncer/

  header(class="mb-4 space-y-2")
    span(class="block text-4xl") Thom Bruce

    nav(class="space-x-2")
      NuxtLink(to="/") Home
      NuxtLink(to="/about") About

    div
      ColorMode/

  NuxtPage/
</template>
