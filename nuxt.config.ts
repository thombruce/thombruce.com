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
  ],
})
