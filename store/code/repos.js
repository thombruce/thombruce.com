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
      const repos = await this.$axios.$get('https://api.github.com/users/thombruce/repos?per_page=100&type=public&sort=updated')

      commit('push', { repos })
    }

    return getters.all
  }
}

export const mutations = {
  push (state, { repos }) {
    repos = repos.reduce((obj, item) => {
      obj[item.id] = item
      return obj
    }, {})

    Vue.set(state, 'list', { ...state.list, ...repos })
  }
}
