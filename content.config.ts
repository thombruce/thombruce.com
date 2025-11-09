import { defineContentConfig, defineCollection } from '@nuxt/content'
import { z } from 'zod'

export default defineContentConfig({
  collections: {
    content: defineCollection({
      type: 'page',
      source: {
        include: '**/*',
        exclude: [
          '**/.*',
        ],
      },
      schema: z.object({
        layout: z.enum(['default', 'fountain']).optional(),
        date: z.date(),
      })
    })
  }
})
