<template lang='pug'>
#appContainer.drawer
  input#main-menu.drawer-toggle(type='checkbox' v-model="showMainMenu")

  TntLayoutsModernMain

  TntLayoutsModernSidebar(v-on:close-menu='showMainMenu = false')
</template>

<script>
import { themeChange } from 'theme-change'
export default {
  data () {
    return {
      showMainMenu: false
    }
  },
  mounted () {
    themeChange(false)
  }
}
</script>
