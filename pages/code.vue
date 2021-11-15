<template lang='pug'>
article
  h2 Favourites
  .flex.gap-2
    article.flex-1.card.card-side.bordered.mb-4(v-for='favorite in favorites')
      .card-body
        header
          h2.card-title
            a(:href='favorite.html_url') {{ favorite.name }}
        div(v-if='favorite.description')
          p {{ favorite.description }}

  template(v-if='events.length')
    h2 Activity
    .table-container
      table
        thead
          tr
            th Type
            th Repo
            th Author
            th Time
        tbody
          tr(v-for='(event, i) in events')
            td {{ event.type }}
            td
              a(:href="'https://github.com/' + event.repo.name") {{ event.repo.name }}
            td
              .avatar.mr-1
                .mb-8.rounded-full.w-6.h-6
                  img.inline-block.align-middle(:src='event.actor.avatar_url')
              span.inline-block.align-middle {{ event.actor.login }}
            td {{ event.created_at }}

  h2 Repositories
  .table-container
    table
      thead
        tr
          th Name
          th Description
          th Language
          th.text-center Git
          th.text-center Issues
      tbody
        tr(v-for='(repo, i) in repos')
          th
            a(v-if='repo.homepage' :href='repo.homepage') {{ repo.name }}
            span(v-else) {{ repo.name }}
          td {{ repo.description }}
          td {{ repo.language }}
          td.text-center
            a.text-lg(:href='repo.html_url')
              fa(:icon='faCodeBranch')
          td.text-center
            a.text-lg(:href="repo.html_url + '/issues'")
              .indicator
                .indicator-item.badge.badge-error {{ repo.open_issues_count }}
                fa(:icon='faBug')
</template>

<script>
import { faCodeBranch, faBug } from '@fortawesome/free-solid-svg-icons'

export default {
  layout: 'modern',
  data () {
    return {
      events: []
    }
  },
  async asyncData ({ store }) {
    const favorites = await store.dispatch('code/favorites/fetchAll')
    const repos = await store.dispatch('code/repos/fetchAll')

    return { favorites, repos }
  },
  async created () {
    this.events = await this.$store.dispatch('code/events/fetchAll')
  },
  computed: {
    faCodeBranch () {
       return faCodeBranch
    },
    faBug () {
      return faBug
    }
  },
  head () {
    return {
      titleTemplate: null
    }
  }
}
</script>

<style lang='postcss' scoped>
.table-container {
  @apply w-full h-full overflow-auto;
  & table {
    @apply table w-full h-full;
    & th {
      &:first-child {
        @apply relative sm:sticky;
      }
    }
    & thead {
      & th {
        @apply sticky;
        top: 0;
        z-index: 20;
        &:first-child {
          @apply sticky left-auto sm:left-0;
          z-index: 30;
        }
      }
    }
    & thead, & tfoot {
      & th, & td {
        &:first-child {
          @apply rounded-l-none;
        }
        &:last-child {
          @apply rounded-r-none;
        }
      }
    }
    & td {
      @apply whitespace-normal;
    }
  }
}
</style>
