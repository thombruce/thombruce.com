<script setup lang="ts">
const weekdays = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]

const today = ref(new Date())

const weekday = ref(weekdays[today.value.getDay()])
</script>

<template lang="pug">
time(:datetime="today.toISOString().substring(0, 10)") {{ weekday }}
</template>
