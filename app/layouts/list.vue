<script setup lang="ts">
const route = useRoute()

const { data: pages } = await useAsyncData(
  `${route.path}-list`,
  () => queryCollection('content')
    .where('path', 'LIKE', `${route.path}%`)
    // .where('path', 'NOT LIKE', `${route.path}%/%`)
    .all()
)
</script>

<template lang="pug">
article(class="prose dark:prose-invert max-w-none")
  slot/
  ol(v-if="pages?.length")
    li(v-for="page in pages")
      NuxtLink(:to="page.path") {{ page.title }}
  div(v-else)
    h1 No Posts Found
</template>
