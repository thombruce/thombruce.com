# ThomBruce.com

## Writing Content

Both of the below should be equivalent:

```md
# Hello, World!

Lorem ipsum dolor sit amet.
```

```md
---
title: Hello, World!
---

Lorem ipsum dolor sit amet.
```

The theme sets `display: none;` on the first child of the document if it is an `h1` element.
The title is otherwise rendered outside of the `<ContentRenderer>` in a separate header.

<!--
# Nuxt Minimal Starter

Look at the [Nuxt documentation](https://nuxt.com/docs/getting-started/introduction) to learn more.

## Setup

Make sure to install dependencies:

```bash
# npm
npm install

# pnpm
pnpm install

# yarn
yarn install

# bun
bun install
```

## Development Server

Start the development server on `http://localhost:3000`:

```bash
# npm
npm run dev

# pnpm
pnpm dev

# yarn
yarn dev

# bun
bun run dev
```

## Production

Build the application for production:

```bash
# npm
npm run build

# pnpm
pnpm build

# yarn
yarn build

# bun
bun run build
```

Locally preview production build:

```bash
# npm
npm run preview

# pnpm
pnpm preview

# yarn
yarn preview

# bun
bun run preview
```

Check out the [deployment documentation](https://nuxt.com/docs/getting-started/deployment) for more information.
-->
