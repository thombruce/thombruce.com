import { Resend } from 'resend'

import mjml2html from 'mjml'

import fs from 'fs'
import path from 'path'

import { v4 as uuid } from 'uuid'

const resendApiKey = process.env.NUXT_RESEND_API_KEY

const resend = new Resend(resendApiKey)

// TODO: With config now set in a separate file, this is now ready
//       to be moved to TNT (Static).
// IDEA: We should integrate at least mjml or whatever it's called
//       before this. And we might want to adjust some other things
//       here first too.
// E.G.: We might want to add checkbox options to the form submission
//       before integrating MJML. That way we have a sense of how these
//       are passed through before we refactor for presentation.
//       And then we would look at generalising such that we can migrate
//       this file to TNT... if that makes any sense to do.
//       I think it does, but it absolutely needs generalisation first.
//       It can have no expectations about the contents of the form,
//       but should adapt to present the contents it is given in an
//       easy to read manner... and ideally in the order given?

function fromLineMjml(name, email?) {
  if (name && email)
    return `<strong>From:</strong> <a href="mailto:${email}">${name} (${email})</a>`
  return `<strong>From:</strong> <span>${name}</span>`
}

function fromLineText(name, email?) {
  if (name && email)
    return `From: ${name} (${email})`
  return `From: ${name}`
}

export default async function handler(req: Request, res) {
  // Config
  const configPath = path.join(process.cwd(), 'api.config.json')
  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'))

  const { body } = req as any

  const data = await resend.emails.send({
    from: config.from,
    to: [config.to],
    subject: body.subject,
    html: mjml2html(`
      <mjml>
        <mj-body>
          <mj-section>
            <mj-column>
              <mj-text>
                ${fromLineMjml(body.name, body.email)}
              </mj-text>
              <mj-text>
                <h2>Message</h2>
                <pre>${body.message}</pre>
              </mj-text>
            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>
    `).html,
    text: `
      ${fromLineText(body.name, body.email)}

      ${body.message}
    `,
    headers: {
      // Prevents threading in GMail
      'X-Entity-Ref-ID': uuid(),
    },
  })
  // TODO: Handle errors from Resend

  // TODO: Generalise for use as...
  //       - Enquiries form
  //       - Request for quote / cost breakdown
  //       - Notifications (e.g. I want to be notified when somebody submits to the guestbook)

  return res.send(data)
}
