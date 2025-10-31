<script setup lang="ts">
import { computed, useSiteConfig } from '#imports'
import { parseURL } from 'ufo'

const props = withDefaults(defineProps<{
  title?: string
  website?: string
  color?: string
}>(), {
  title: 'title',
  website: undefined,
  color: undefined,
})

// fallback to site name
const website = computed(() => {
  return props.website || parseURL(useSiteConfig().url).host
})

// fallback to config color
const color = computed(() => {
  return props.color || '#09090b'
})
</script>

<template>
  <div class="h-full w-full flex items-start justify-start text-white" :class="`bg-[${color}]`">
    <div class="flex items-start justify-start h-full">
      <div class="flex flex-col justify-between w-full h-full">
        <h1 class="text-[80px] p-20 font-black text-left"
          style="display: block; line-clamp: 2; text-overflow: ellipsis;">
          {{ title }}
        </h1>
        <p class="text-2xl pb-10 px-20 font-bold mb-0" style="display: block; line-clamp: 3; text-overflow: ellipsis;">
          {{ website }}
        </p>
      </div>
    </div>
  </div>
</template>
