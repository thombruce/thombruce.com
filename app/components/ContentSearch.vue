<script setup lang="js">
// TODO: At present this only searches Markdown.
// I'd love to find a way to include my Fountain screenplays in the search results.

import Fuse from 'fuse.js'

const query = ref('')
const { data } = await useAsyncData('content-search', () => queryCollectionSearchSections('content'))

const fuse = new Fuse(data.value, {
  keys: ['title', 'content']
})

const result = computed(() =>
  fuse.search(toValue(query))
    .filter(r => !/\/#/.test(r.item.id)) // Discard subheadings on home page
    .filter(r => !/(.*?)#\1$/.test(r.item.id)) // Discard duplicates (where subheading matches title)
  // .slice(0, 10) // Return only first 10 results
)
</script>

<template lang="pug">
ModalView(buttonText="Search")
  form
    label(for="query" class="text-3xl font-bold font-pixel") Search
    input(id="query" v-model="query" class="block w-full border p-1")
  div(v-if="result?.length" class="not-prose py-4")
    ol(class="max-h-[250px] overflow-scroll space-y-4")
      li(v-for="page in result" class="max-w-md")
        NuxtLink(:to="page.item.id" class="link-to-post")
          h2(class="text-2xl font-bold font-pixel") {{ page.item.title }}
          p(class="max-h-[100px] overflow-hidden text-sm text-zinc-700 dark:text-zinc-300") {{ page.item.content }}
</template>
