<template lang='pug'>
div.py-10
  p.text-center.text-4xl
    RockBandSimpleIcon(icon='ethereum')
    | {{ ether }}
</template>

<script>
export default {
  async asyncData ({ $Web3 }) {
    const web3 = new $Web3('https://mainnet.infura.io/v3/a185cad5258f4c89a0107035403eaa4e')

    const wei = await web3.eth.getBalance('0x98cCAac584a012D27A67ABD1586868a1bc3b33e3')
    const ether = web3.utils.fromWei(wei)

    return { ether }
  }
}
</script>
