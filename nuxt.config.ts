import tailwindcss from '@tailwindcss/vite'

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },

  nitro: {
    static: true,
  },

  vite: {
    plugins: [
      tailwindcss(),
    ]
  },

  modules: ['@nuxtjs/color-mode', '@nuxt/content', '@nuxt/fonts'],

  css: [
    './assets/css/main.css',
    '@thombruce/fountainjs/src/fountain.css',
  ],

  routeRules: {
    '/discord': { redirect: 'https://discord.gg/SAUagUgTfa' },
    '/github': { redirect: 'https://github.com/thombruce' },
    '/linkedin': { redirect: 'https://www.linkedin.com/in/thombruce' },
    '/bluesky': { redirect: 'https://bsky.app/profile/thombruce.com' },
    '/mastodon': { redirect: 'https://mas.to/@thombruce' },
    '/twitch': { redirect: 'https://twitch.tv/thombruce' },
    '/youtube': { redirect: 'https://youtube.com/@thombruce' },
  },

  content: {
    build: {
      transformers: [ // See: https://content.nuxt.com/docs/advanced/transformers
        '~~/transformers/fountain',
      ],
    },
  },
})
