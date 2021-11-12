<template lang='pug'>
div
  template(v-if='articles')
    JournalCard(v-for='article in articles' :key='article.slug' :article='article')
  div.text-center(v-else)
    span.text-2xl.text-secondary There's nothing to see yet.
</template>

<script>
export default {
  props: [
    'articles'
  ]
}
</script>
