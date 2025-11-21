<script setup lang="ts">
const uid = useId()

const { buttonText = "Open" } = defineProps<{
  buttonText?: String
}>()
</script>

<template lang="pug">
div
  button(:popovertarget="`modal-${uid}`" class="cursor-pointer") {{ buttonText }}

  div(
    popover
    :id="`modal-${uid}`"
    class="m-auto px-4 py-2 bg-light dark:bg-dark text-dark dark:text-light border"
  )
    slot/
</template>
