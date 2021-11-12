<template lang='pug'>
article.card.card-side.bordered.mb-4
  figure
    NuxtImg(v-if='article.image' :src='article.image' width='200' height='200' style='width:200px;height:200px;object-fit:cover;')

  .card-body
    header
      h2.card-title(v-if='article.title')
        NuxtLink(:to='article') {{ article.title }}
      time(v-if='article.date' :datetime='article.date') {{ article.date | toLocaleString }}
    TntContentBody(:article='article')
</template>

<script>
export default {
  props: [
    'article'
  ]
}
</script>
