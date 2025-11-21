<script setup lang="ts">
const popover: Ref = ref(null)

const { buttonText = "Open" } = defineProps<{
  buttonText?: String
}>()

const emit = defineEmits(['opened'])

const openModal = () => {
  popover.value.showPopover()
  emit('opened')
}

// Close the popover on navigation (if a link is clicked)
const route = useRoute()
watch(() => route.fullPath, (_newPath, _oldPath) => {
  popover.value.hidePopover()
})
</script>

<template lang="pug">
div
  button(@click="openModal()" class="cursor-pointer") {{ buttonText }}

  div(
    popover
    ref="popover"
    class="max-w-full md:max-w-1/2 lg:max-w-1/3 xl:max-w-1/4 m-auto px-4 py-2 bg-light dark:bg-dark text-dark dark:text-light border"
  )
    slot/
</template>
