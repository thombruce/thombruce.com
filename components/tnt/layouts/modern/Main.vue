<template lang='pug'>
.flex-grow.block.overflow-x-hidden.bg-base-100.text-base-content.drawer-content
  .sticky.inset-x-0.top-0.z-50.w-full.py-1.transition.duration-200.ease-in-out.border-b.border-base-200.bg-base-100.text-base-content
    .mx-auto.space-x-1.navbar.max-w-none
      .navbar-start
        NuxtLink.btn.btn-ghost(to='/')
          span.text-lg.font-bold {{ title }}
      .navbar-end
        .hidden(v-for='(collection, dir) in collections' class='lg:inline')
          NuxtLink.btn.btn-ghost(v-if="dir != '/'" :to='dir') {{ dir.split('/').pop() | titleize }}
          NuxtLink.btn.btn-ghost(v-else v-for='page in collection' :key='page.slug' :to='page') {{ page.title }}
        TntUIThemeToggle
        label.btn.btn-square.btn-ghost.drawer-button(for='main-menu' class='lg:hidden')
          fa(:icon='faBars')

  main.max-w-prose.mx-auto
    TntUIBreadcrumbs
    Nuxt

  TntLayoutsClassicFooter
</template>

<script>
import { groupBy } from 'lodash'
import { faBars } from '@fortawesome/free-solid-svg-icons'

export default {
  data () {
    return {
      title: this.$nuxt.$options.head.title,
      collections: null
    }
  },
  computed: {
    faBars () {
       return faBars
    }
  },
  async fetch() {
    const nestedPages = await this.$content({ deep: true })
      .where({ draft: { $ne: true }})
      .sortBy('dir')
      .sortBy('order')
      .sortBy('title')
      .fetch()
      .catch(() => {})
    this.collections = groupBy(nestedPages, 'dir')
  },
}
</script>