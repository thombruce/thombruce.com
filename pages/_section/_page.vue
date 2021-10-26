<template lang='pug'>
div
  TntContent(v-if='!Array.isArray(post)' :article='post')
  article(v-else)
    header
      h1 {{ slug | titleize }}
    TntBlogList(:articles='post')
</template>

<script>
export default {
  async asyncData ({ $content, $taxonomies, params }) {
    const slug = params.page

    const post = await $content(params.section, slug)
      .where({ draft: { $ne: true } })
      .fetch()
      .catch(async () => {
        const taxonomy = params.section
        const term = await $taxonomies(taxonomy, '', { deep: true }).find(slug)
        const articles = await $content('', { deep: true })
          .where({
            $and: [
              { draft: { $ne: true } },
              { $or: [{ [taxonomy]: { $contains: term.title } }, { [taxonomy]: { $eq: term.title } }] }
            ]
          })
          .sortBy('date', 'desc')
          .fetch()
        
        return articles
      })

    return { post }
  },
  head () {
    return {
      title: this.post.title
    }
  }
}
</script>
