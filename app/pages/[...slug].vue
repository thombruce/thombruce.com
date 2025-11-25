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

    div(v-if="page?.tags?.length" class="not-prose")
      h2 Tags
      ul(class="space-x-2")
        li(v-for="tag in page.tags" class="inline-block bg-teal-400 dark:bg-teal-600 rounded px-2 font-bold before:content-['#']") {{ tag }}

    div(v-if="page?.contexts?.length" class="not-prose")
      h2 Mentions
      ul(class="space-x-2")
        li(v-for="context in page.contexts" class="inline-block bg-rose-400 dark:bg-rose-600 rounded px-2 font-bold before:content-['@']") {{ context }}

    div(v-if="page?.projects?.length" class="not-prose")
      h2 Projects
      ul(class="space-x-2")
        li(v-for="project in page.projects" class="inline-block bg-amber-400 dark:bg-amber-600 rounded px-2 font-bold before:content-['+']") {{ project }}

  ContentRenderer(v-if="page" :value="page" class="content-renderer")/

  div(v-else)
    h1 Page Not Found
</template>

<style lang="postcss">
.content-renderer h1:first-child {
  display: none;
}
</style>
