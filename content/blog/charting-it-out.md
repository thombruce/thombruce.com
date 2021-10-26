---
title: Charting it Out
description: 'Adding charts to my Markdown documents'
authors:
  - Thom Bruce
date: 2021-05-21T15:56:42Z
updatedAt: 2021-05-28T18:08:33Z
categories:
  - Jamstack
series:
  - Getting Started
tags:
  - Nuxt.js
  - Markdown
  - Charts
---

Following on from adding KaTeX math support and exploring the capabilities of Markdown in my previous post, I realised a big omission from the tools I might like to employ is charts. I've worked with charts before using a number of different libraries. My favourite, simply because it appears to be the most powerful and flexible, is [D3.js](https://d3js.org/). Unfortunately, because it is so powerful and flexible, it is not easy to drop in and use straight away - I would need to do a lot of custom configuration. So, as a simpler measure for the time being, I've opted to install the beautiful little [Chart.js](https://www.chartjs.org/) library.

**Installation:**

```sh
yarn add vue-chartjs chart.js@2.9.4
```

**Component:**

```vue[BarChart.vue]
<script>
import { Bar } from 'vue-chartjs'

export default {
  extends: Bar,
  props: [
    'labels',
    'datasets'
  ],
  mounted () {
    this.renderChart({
      labels: this.labels,
      datasets: this.datasets
    })
  }
}
</script>
```

**Usage:**

```md
<bar-chart :labels='[`One`, `Two`, `Three`]' :datasets='[{ label: `Data`, backgroundColor: [`#EF4444`, `#3B82F6`, `#10B981`], data: [40, 20, 12] }]'></bar-chart>
```

That's a simple, drop-in and start using solution to a problem I don't need to give a lot of consideration just yet.

The library is lacking in some advanced features, but many of these can be added through plugins - I'm just not really sure how to go about that using the Vue package yet. I also think I could clean up the manner in which I pass data to the component - it could be simplified, more intuitive. Chart.js may need the data in this format, but I can simplify the props required for end users; one big thing I may add is a series of default colours that are selected automatically. But for now, this will do.

Using this library, I now have access to the following charts, and could expand a little on this list in the future:

<div class='flex flex-wrap'>
  <div class='w-full md:w-1/2 md:p-5'>
  <h2>Bar Chart</h2>

  <playfair-bar-chart :labels='[`One`, `Two`, `Three`]' :datasets='[{ label: `Data`, backgroundColor: [`#EF4444`, `#3B82F6`, `#10B981`], data: [40, 20, 12] }]'></playfair-bar-chart>
  </div>

  <div class='w-full md:w-1/2 md:p-5'>
  <h2>Horizontal Bar Chart</h2>

  <playfair-horizontal-bar-chart :labels='[`One`, `Two`, `Three`]' :datasets='[{ label: `Data`, backgroundColor: [`#EF4444`, `#3B82F6`, `#10B981`], data: [40, 20, 12] }]'></playfair-horizontal-bar-chart>
  </div>

  <div class='w-full md:w-1/2 md:p-5'>
  <h2>Line Chart</h2>

  <playfair-line-chart :labels='[`One`, `Two`, `Three`]' :datasets='[{ label: `Data`, borderColor: `rgba(59, 130, 246, 1)`, backgroundColor: `rgba(59, 130, 246, .25)`, fill: true, data: [40, 20, 12] }]'></playfair-line-chart>
  </div>

  <div class='w-full md:w-1/2 md:p-5'>
  <h2>Pie Chart</h2>

  <playfair-pie-chart :labels='[`One`, `Two`, `Three`]' :datasets='[{ label: `Data`, borderWidth: 0, backgroundColor: [`#EF4444`, `#3B82F6`, `#10B981`], data: [40, 20, 12] }]'></playfair-pie-chart>
  </div>

  <div class='w-full md:w-1/2 md:p-5'>
  <h2>Doughnut Chart</h2>

  <playfair-doughnut-chart :labels='[`One`, `Two`, `Three`]' :datasets='[{ label: `Data`, borderWidth: 0, backgroundColor: [`#EF4444`, `#3B82F6`, `#10B981`], data: [40, 20, 12] }]'></playfair-doughnut-chart>
  </div>

  <div class='w-full md:w-1/2 md:p-5'>
  <h2>Polar Area Chart</h2>

  <playfair-polar-area-chart :labels='[`One`, `Two`, `Three`]' :datasets='[{ label: `Data`, borderColor: [`rgba(239, 67, 68, 1)`, `rgba(39, 185, 129, 1)`, `rgba(59, 130, 246, 1)`], backgroundColor: [`rgba(239, 67, 68, .25)`, `rgba(39, 185, 129, .25)`, `rgba(59, 130, 246, .25)`], data: [40, 20, 12] }]'></playfair-polar-area-chart>
  </div>

  <div class='w-full md:w-1/2 md:p-5'>
  <h2>Radar Chart</h2>

  <playfair-radar-chart :labels='[`One`, `Two`, `Three`]' :datasets='[{ label: `Data`, borderColor: `rgba(59, 130, 246, 1)`, backgroundColor: `rgba(59, 130, 246, .25)`, data: [40, 20, 12] }]'></playfair-radar-chart>
  </div>

  <div class='w-full md:w-1/2 md:p-5'>
  <h2>Scatter Chart</h2>

  <playfair-scatter-chart :datasets='[{ label: `Data`, backgroundColor: `#EF4444`, data: [{ x: 20, y: 30 }, { x: 40, y: 10 }] }, { label: `Data2`, backgroundColor: `#3B82F6`, data: [{ x: 35, y: 15 }] }]'></playfair-scatter-chart>
  </div>

  <div class='w-full md:w-1/2 md:p-5'>
  <h2>Bubble Chart</h2>

  <playfair-bubble-chart :datasets='[{ label: `Data`, borderColor: `rgba(239, 67, 68, 1)`, backgroundColor: `rgba(239, 67, 68, .25)`, data: [{ x: 20, y: 30, r: 20 }, { x: 40, y: 10, r: 10 }] }, { label: `Data2`, borderColor: `rgba(59, 130, 246, 1)`, backgroundColor: `rgba(59, 130, 246, .25)`, data: [{ x: 35, y: 15, r: 50 }] }]'></playfair-bubble-chart>
  </div>
</div>

_Update (2021-05-28): I've now packaged these charts into my own custom library, [Vue Playfair](https://github.com/thombruce/vue-playfair)._