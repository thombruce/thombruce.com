<template lang='pug'>
TntContent(:article='post')
</template>

<script>
export default {
  async asyncData ({ $content, params }) {
    const post = await $content(params.section, params.page)
      .where({ draft: { $ne: true } })
      .fetch()
      .catch(() => {})

    return { post }
  },
  head () {
    return {
      title: this.post.title
    }
  }
}
</script>
