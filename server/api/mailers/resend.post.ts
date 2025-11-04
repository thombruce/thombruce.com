import { z } from 'zod'
import { Resend } from 'resend'

const { resendApiKey } = useRuntimeConfig()

const resend = new Resend(resendApiKey || 'Please provide API key')

const schema = z.object({
  name: z.string(),
  email: z.string().email(),
  subject: z.string(),
  message: z.string(),
})

export default defineEventHandler(async (event) => {
  try {
    const body = await readValidatedBody(event, body => schema.safeParse(body))

    if (!body.success)
      throw body.error.issues

    const { name, email, subject, message } = body.data

    const data = await resend.emails.send({
      from: 'ThomBruce.com <hello@mail.thombruce.com>', // TODO: Set in config
      to: ['thom@thombruce.com'], // TODO: Set in config
      subject: `ThomBruce.com - ${subject}`, // TODO: Allow subject to be selectable; obtain as body.subject
      html: `
<strong>From:</strong> <span>${name} (${email})</span>
<p>${message}</p>
      `,
      text: `
From: ${name} (${email})

${message}
      `,
    })

    return data
  } catch (error) {
    return { error }
  }
})
