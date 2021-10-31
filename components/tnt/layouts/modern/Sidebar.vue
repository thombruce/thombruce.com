<template lang='pug'>
.drawer-side
  label.drawer-overlay(for='main-menu')
  aside.flex.flex-col.justify-between.border-r.border-base-200.bg-base-100.text-base-content.w-80
    div.h-full
      ul.menu.flex.flex-col.p-4
        template(v-for='(collection, dir) in collections')
          li(v-if="dir != '/'")
            NuxtLink(v-on:click.native="$emit('close-menu')" :to='dir') {{ dir.split('/').pop() | titleize }}
          li(v-else v-for='page in collection' :key='page.slug')
            NuxtLink(v-on:click.native="$emit('close-menu')" :to='page') {{ page.title }}
</template>

<script>
import { groupBy } from 'lodash'

export default {
  data () {
    return {
      collections: null
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
