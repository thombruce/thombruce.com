<script setup lang="ts">
import * as z from 'zod'

const subjects = ['Information', 'Issue'] as const

const schema = z.object({
  name: z.string(),
  email: z.string().email(),
  subject: z.enum(subjects),
  message: z.string(),
})
type Schema = z.output<typeof schema>

const state = ref<Partial<Schema>>({
  name: '',
  email: '',
  subject: 'Information',
  message: '',
})

function clear() {
  state.value = { name: '', email: '', subject: 'Information', message: '' }
}

async function submit() {
  // TODO: Handle validation errors
  schema.parse(state.value)

  await $fetch('/api/mailers/resend', {
    method: 'POST',
    body: state.value,
  })

  clear()
}
</script>

<template lang="pug">
NuxtLayout
  h1(class="text-4xl font-bold font-pixel") Contact 

  form(@submit.prevent="submit" class="space-y-4")
    div
      label(for="name") Name
      input(v-model="state.name" id="name" class="block w-full border p-1")/
  
    div
      label(for="email") Email
      input(v-model="state.email" id="email" class="block w-full border p-1")/
  
    div
      label(for="subject") Subject
      select(v-model="state.subject" id="subject" class="block w-full border p-1")
        option(v-for="subject in subjects") {{ subject }}
  
    div
      label(for="message") Message
      textarea(v-model="state.message" id="message" class="block w-full border p-1")/
  
    div(class="flex flex-row-reverse justify-between")
      button(type="submit" class="border rounded-lg p-2 cursor-pointer") Submit
      button(@click="clear" class="border rounded-lg border-red-500 p-1 text-sm text-red-500 cursor-pointer") Clear
</template>
