<template lang='pug'>
div.py-10
  p.text-center.text-4xl
    RockBandSimpleIcon(icon='ethereum')
    | {{ balance }}
</template>

<script>
export default {
  data () {
    return {
      web3: null,
      wei: null
    }
  },
  computed: {
    balance () {
      if (this.wei) return this.web3.utils.fromWei(this.wei)
    }
  },
  mounted () {
    this.initWeb3()
    this.fetchBalance()
  },
  methods: {
    initWeb3 () {
      this.web3 = new this.$Web3(this.$Web3.givenProvider || 'https://mainnet.infura.io/v3/a185cad5258f4c89a0107035403eaa4e')
    },
    async fetchBalance () {
      this.wei = await this.web3.eth.getBalance('0x98cCAac584a012D27A67ABD1586868a1bc3b33e3')
    }
  }
}
</script>
