import Vue from 'vue'
import { isEmpty } from 'lodash'

export const state = () => ({
  list: {}
})

export const getters = {
  all: (state, _getters, _rootState, _rootGetters) => {
    const list = state.list
    if (list) {
      return Object.values(list)
    }
  }
}

export const actions = {
  async fetchAll ({ commit, state, getters }) {
    if (isEmpty(state.list)) {
      let favorites = await this.$axios.$get('https://api.github.com/users/thombruce/starred?per_page=100')
      favorites = favorites.filter(favorite => favorite.owner.login === 'thombruce')

      commit('push', { favorites })
    }

    return getters.all
  }
}

export const mutations = {
  push (state, { favorites }) {
    favorites = favorites.reduce((obj, item) => {
      obj[item.id] = item
      return obj
    }, {})

    Vue.set(state, 'list', { ...state.list, ...favorites })
  }
}
