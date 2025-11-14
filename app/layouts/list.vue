<script setup lang="ts">
const route = useRoute()

const { data: pages } = await useAsyncData(
  `${route.path}-list`,
  () => queryCollection('content')
    .where('path', 'LIKE', `${route.path}%`)
    .where('path', 'NOT LIKE', `${route.path}/%/%`)
    .where('path', 'NOT LIKE', `${route.path}`)
    .order('date', 'DESC')
    .all()
)
</script>

<template lang="pug">
article(class="prose dark:prose-invert max-w-none")
  slot/
  div(v-if="pages?.length" class="not-prose")
    ol(class="space-y-4")
      li(v-for="page in pages")
        NuxtLink(:to="page.path" class="link-to-post")
          h2(class="text-3xl") {{ page.title }}
          p {{ page.description }}
  div(v-else)
    h1 No Posts Found
</template>

<style lang="postcss">
/*
   This first implementation looks correct
   but isn't working. It is as if everything
   up to 'dark' is being treated as the property,
   instead of the apply rule just working.
   TODO: Investigate
 */
a.link-to-post:hover h2 {
  /* @apply text-zinc-800 dark:text-zinc-200; */
  color: var(--color-zinc-900);
}

.dark a.link-to-post:hover h2 {
  /* @apply text-zinc-800 dark:text-zinc-200; */
  color: var(--color-zinc-100);
}
</style>
