import { type ContentFile, defineTransformer } from '@nuxt/content'

import { visit, textContent } from 'minimark'

// NOTE: We could consider using the more explicit hast syntax instead
// of the generated minimark AST.
// import { toHast, fromHast } from 'minimark/hast'
// import { visit } from 'unist-util-visit'

export default defineTransformer({
  name: 'Markdown',
  extensions: ['.md'],
  transform: (file: any) => {
    const {
      id,
      body,
      date,
      tags = [],
      contexts = [],
      projects = []
    }: {
      id: string,
      body: any,
      date: Date,
      tags: string[],
      contexts: string[],
      projects: string[]
    } = file

    // TODO: Check first whether or not the file already has a date
    //       (it could be provided in Markdown header).
    //       If so, the below is unnecessary.
    // TODO: Also, since we are using a regex anyway, could this be
    //       more efficient if we wrote the whole operation as a single
    //       .replace() function?
    const seq_id = id.split('/').pop().split('.')[0]
    const date_from_seq_id = seq_id.length === 8 ? new Date(seq_id.replace(/(\d{4})(\d{2})(\d{2})/, "$1-$2-$3")) : null

    // TODO: Add support for headings
    // TODO: Add support for blockquotes
    // TODO: Ensure that hashtags in inline code are not included
    visit(body, (node) => ['p'].includes(node[0]), (node) => {
      tags.push(...(textContent(node).match(/(?<=(?:^|\s)#)\w+/g) || []))
      contexts.push(...(textContent(node).match(/(?<=(?:^|\s)(?:\+\w+)?@)\w+/g) || []))
      projects.push(...(textContent(node).match(/(?<=(?:^|\s)\+)\w+(?:@\w+)?/g) || []))
    })

    return {
      ...file,

      date: date || date_from_seq_id,
      tags,
      contexts,
      projects,
    }
  },
})
