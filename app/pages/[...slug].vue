<script setup lang="ts">
import type { LayoutKey } from '#build/types/layouts'

const route = useRoute()

const { data: page } = await useAsyncData(
  route.path,
  () => queryCollection('content').path(route.path).first()
)

const layout = (page.value?.layout || 'default') as LayoutKey

useSeoMeta({
  title: page.value?.title,
  description: page.value?.description
})

// definePageMeta({
//   colorMode: 'dark',
// })

defineOgImageComponent('OgColor')
</script>

<template lang="pug">
NuxtLayout(:name="layout" :page="page")
  div(class="prose dark:prose-invert")
    ContentRenderer(v-if="page" :value="page")/
    div(v-else)
      h1 Page Not Found
</template>
