
<template lang='pug'>
article
  TntBlogList(v-if='posts' :articles='posts')
</template>

<script>
export default {
  async asyncData ({ $content }) {
    let posts = await $content('blog')
      .where({ draft: { $ne: true } })
      .sortBy('date', 'desc')
      .fetch()
      .catch(() => {})

    return { posts }
  }
}
</script>
