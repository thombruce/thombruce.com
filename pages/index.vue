
<template lang='pug'>
article
  TntBlogList(v-if='content' :articles='content')
</template>

<script>
export default {
  layout: 'modern',
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
