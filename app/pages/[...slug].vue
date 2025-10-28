<script setup lang="ts">
const route = useRoute()

const { data: page } = await useAsyncData(
  route.path,
  () => queryCollection('content').path(route.path).first()
)

useSeoMeta({
  title: page.value?.title,
  description: page.value?.description
})

// definePageMeta({
//   colorMode: 'dark',
// })
</script>

<template lang="pug">
div(class="prose dark:prose-invert")
  ContentRenderer(v-if="page" :value="page")/
  div(v-else)
    h1 Page Not Found
</template>
