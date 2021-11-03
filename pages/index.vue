
<template lang='pug'>
div
  .hero.min-h-screen.bg-base-200
    .hero-content(class='flex-col lg:flex-row-reverse')
      figure
        NuxtImg(src='anywhere.png' width='400')
        figcaption.text-center.text-xs
          a(href="https://storyset.com/people" target='_blank') Illustration by Storyset
      div
        h1 Hi there, I'm Thom!
        p Welcome to thombruce.com.
  article.max-w-prose.mx-auto
    TntBlogList(v-if='content' :articles='content')
</template>

<script>
export default {
  layout: 'home',
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
