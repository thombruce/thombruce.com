<template lang='pug'>
JournalCard(:article='content')
</template>

<script>
export default {
  layout: 'modern',
  async asyncData ({ $content, params }) {
    const slug = params.page

    const content = await $content('journal', slug)
      .where({ draft: { $ne: true } })
      .fetch()
      .catch(async () => {})

    return { content }
  },
  head () {
    return {
      title: this.content.title
    }
  }
}
</script>
