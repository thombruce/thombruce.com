<template lang='pug'>
div
  TntContent(v-if='!Array.isArray(content)' :article='content')
  article(v-else)
    header
      h1 {{ slug | titleize }}
    TntBlogList(:articles='content')
</template>

<script>
export default {
  async asyncData ({ $content, $taxonomies, params }) {
    const slug = params.page

    const content = await $content(slug)
      .where({ draft: { $ne: true } })
      .sortBy('date', 'desc')
      .fetch()
      .catch(async () => {
        const terms = await $taxonomies(slug, '', { deep: true }).all()
        return terms
      })

    return { slug, content }
  }
}
</script>