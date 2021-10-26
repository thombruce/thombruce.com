<template lang='pug'>
article
  header
    h1 {{ slug | titleize }}
  TntBlogList(:articles='posts')
</template>

<script>
export default {
  async asyncData ({ $content, params }) {
    const slug = params.page

    const posts = await $content(slug)
      .where({ draft: { $ne: true } })
      .sortBy('date', 'desc')
      .fetch()
      .catch(async () => {})

    return { slug, posts }
  }
}
</script>