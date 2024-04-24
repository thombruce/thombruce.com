// https://nuxt.com/docs/api/configuration/nuxt-config

export default defineNuxtConfig({
  devtools: { enabled: true },
  extends: [
    '@thombruce/tnt-content'
  ],
  build: {
    transpile: [
      'strip-ansi',
      'string-width',
    ],
  }
})