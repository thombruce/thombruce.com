import { type ContentFile, defineTransformer } from '@nuxt/content'

export default defineTransformer({
  name: 'Markdown',
  extensions: ['.md'],
  transform: (file: ContentFile) => {
    const { id } = file

    // TODO: Check first whether or not the file already has a date
    //       (it could be provided in Markdown header).
    //       If so, the below is unnecessary.
    // TODO: Also, since we are using a regex anyway, could this be
    //       more efficient if we wrote the whole operation as a single
    //       .replace() function?
    const seq_id = id.split('/').pop().split('.')[0]
    const date_from_seq_id = seq_id.length === 8 ? new Date(seq_id.replace(/(\d{4})(\d{2})(\d{2})/, "$1-$2-$3")) : null

    return {
      ...file,

      date: date_from_seq_id
    }
  },
})
