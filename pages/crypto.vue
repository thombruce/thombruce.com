<template lang='pug'>
div.py-10
  p.text-center.text-4xl
    RockBandSimpleIcon(icon='ethereum')
    CandyFlossWalletBalance(
      address='0x98cCAac584a012D27A67ABD1586868a1bc3b33e3'
      httpProvider='https://mainnet.infura.io/v3/a185cad5258f4c89a0107035403eaa4e'
    )
</template>

<script>
export default {
}
</script>
