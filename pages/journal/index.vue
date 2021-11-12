<template lang='pug'>
article
  header
    h1 {{ slug | titleize }}
  JournalList(:articles='content')
</template>

<script>
export default {
  layout: 'modern',
  async asyncData ({ $content }) {
    const slug = 'journal'

    const content = await $content(slug)
      .where({ draft: { $ne: true } })
      .sortBy('date', 'desc')
      .fetch()
      .catch(async () => {})

    return { slug, content }
  }
}
</script>