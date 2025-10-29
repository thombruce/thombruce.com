<script setup lang="ts">
const colorMode = useColorMode()

const isDark = computed({
  get() {
    return colorMode.value === 'dark'
  },
  set() {
    colorMode.preference = colorMode.value === 'dark' ? 'light' : 'dark'
  }
})
</script>

<template lang="pug">
ClientOnly(v-if="!colorMode?.forced")
  div(class="space-x-2")
    input(
      v-model="isDark"
      id="color-mode"
      type="checkbox"
    )
    label(for="color-mode") Dark mode
  
  template(#fallback)
    div
</template>
