<script lang="ts" setup>
const { page }: { page?: any } = useAttrs()
</script>

<template lang="pug">
//- NOTE: screenplay margin should not be set by fountain.css
//- We are overriding that default here.
.screenplay(style="margin: 0 0;")
  .title-page(v-html="page.body.html.title_page")
  .script(v-html="page.body.html.script")
</template>
