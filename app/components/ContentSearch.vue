<script setup lang="js">
import Fuse from 'fuse.js'

const query = ref('')
const { data } = await useAsyncData('search-data', () => queryCollectionSearchSections('content'))

const fuse = new Fuse(data.value, {
  keys: ['title', 'description']
})

const result = computed(() => fuse.search(toValue(query)).slice(0, 10))
</script>

<template lang="pug">
ModalView(buttonText="Search")
  div
    form
      label(for="query") Search
      input(id="query" v-model="query" class="block w-full border p-1")
    div(v-if="result?.length" class="not-prose")
      ol(class="space-y-4")
        li(v-for="page in result")
          NuxtLink(:to="page.item.path" class="link-to-post")
            h2 {{ page.item.title }}
            p {{ page.item.description }}
</template>
