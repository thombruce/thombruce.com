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
        layout: z.enum(['default', 'list', 'fountain']).optional(),
        date: z.date(),
        tags: z.array(z.string()).optional(),
        contexts: z.array(z.string()).optional(),
        projects: z.array(z.string()).optional(),
      })
    })
  }
})
