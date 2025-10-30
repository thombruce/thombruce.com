import { type ContentFile, defineTransformer } from '@nuxt/content'

import fountain from '@thombruce/fountainjs'

export default defineTransformer({
  name: 'Fountain',
  extensions: ['.fountain'],
  parse: (file: ContentFile) => {
    const { id, body } = file

    const parsed = fountain.parse(body)

    return {
      id,
      body: parsed,

      title: parsed.title,
      date: parsed.date,
      layout: 'fountain',
    }
  },
})
