<script setup lang="ts">
import type { ContentCollectionItem } from '@nuxt/content'

// TODO: We can simplify. Pass page to thelayout instead of list.
//       Page will already have the path, so route is then redundant.

const route = useRoute()

const { list }: { list?: { field?: keyof ContentCollectionItem, direction?: 'ASC' | 'DESC' } } = useAttrs()
const { field = 'id', direction = 'ASC' }: { field?: keyof ContentCollectionItem, direction?: 'ASC' | 'DESC' } = list || {}

const { data: pages } = await useAsyncData(
  `${route.path}-list`,
  () => queryCollection('content')
    .where('path', 'LIKE', `${route.path}%`)
    .where('path', 'NOT LIKE', `${route.path}/%/%`)
    .where('path', 'NOT LIKE', `${route.path}`)
    .order(field, direction)
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
          NuxtTime(v-if="page?.date" :datetime="page.date" class="text-zinc-600 dark:text-zinc-400")
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
