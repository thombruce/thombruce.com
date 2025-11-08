<script lang="ts" setup>
const { page }: { page?: any } = useAttrs()
</script>

<template lang="pug">
.screenplay(class="mx-auto")
  .title-page(v-html="page.body.html.title_page")
  .script(v-html="page.body.html.script")
</template>
