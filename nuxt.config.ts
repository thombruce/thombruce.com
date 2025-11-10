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

  runtimeConfig: {
    // NUXT_RESEND_API_KEY=<key>
    resendApiKey: '',
  },

  site: {
    name: 'Thom Bruce',
    description: 'Professional software developer',
    url: 'https://www.thombruce.com',
  },

  modules: ['@nuxtjs/color-mode', '@nuxt/content', '@nuxt/fonts', 'nuxt-og-image'],

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
    '/matrix': { redirect: 'https://matrix.to/#/@thombruce:matrix.org' },
  },

  colorMode: {
    classSuffix: '',
  },

  content: {
    build: {
      markdown: {
        highlight: {
          theme: {
            default: 'catppuccin-latte',
            dark: 'catppuccin-mocha',
          },
          langs: [
            // Defaults
            'json', 'js', 'ts', 'html', 'css', 'vue', 'shell', 'mdc', 'md', 'yaml',
            // Custom
            'postcss'
          ],
        }
      },
      transformers: [ // See: https://content.nuxt.com/docs/advanced/transformers
        '~~/transformers/markdown',
        '~~/transformers/fountain',
      ],
    },
  },
})
