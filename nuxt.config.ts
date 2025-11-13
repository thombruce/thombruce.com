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
            // See list of themes: https://github.com/shikijs/textmate-grammars-themes/tree/main/packages/tm-themes
            // NOTE: The background is configured separately in ./app/assets/css/main.css
            default: 'catppuccin-latte',
            dark: 'catppuccin-mocha',

            'catppuccin-latte-theme': 'catppuccin-latte',
            'catppuccin-mocha-theme': 'catppuccin-mocha',
            'dracula-theme': 'dracula',
            'kanagawa-dragon-theme': 'kanagawa-dragon',
            'kanagawa-lotus-theme': 'kanagawa-lotus',
            'kanagawa-wave-theme': 'kanagawa-wave',
            'nord-theme': 'nord',
          },
          langs: [
            // See list of grammars: https://github.com/shikijs/textmate-grammars-themes/tree/main/packages/tm-grammars
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
