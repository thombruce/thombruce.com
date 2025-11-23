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
NuxtLayout(:name="layout" :page="page" :list="page.meta?.list")
  header
    h1(class="mb-2") {{ page.title }}

    NuxtTime(v-if="page?.date" :datetime="page.date" class="text-zinc-600 dark:text-zinc-400")

    template(v-if="page?.tags?.length")
      h2 Tags
      ul
        li(v-for="tag in page.tags") {{ tag }}

    template(v-if="page?.contexts?.length")
      h2 Mentions
      ul
        li(v-for="context in page.contexts") {{ context }}

    template(v-if="page?.projects?.length")
      h2 Projects
      ul
        li(v-for="project in page.projects") {{ project }}

  ContentRenderer(v-if="page" :value="page" class="content-renderer")/

  div(v-else)
    h1 Page Not Found
</template>

<style lang="postcss">
.content-renderer h1:first-child {
  display: none;
}
</style>
