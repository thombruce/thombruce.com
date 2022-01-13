
<template lang='pug'>
div
  article.max-w-prose.mx-auto
    TntBlogList(v-if='content' :articles='content')
</template>

<script>
export default {
  async asyncData ({ $content }) {
    let content = await $content('blog')
      .where({ draft: { $ne: true } })
      .sortBy('date', 'desc')
      .fetch()
      .catch(() => {})

    return { content }
  }
}
</script>
