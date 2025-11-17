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

> [!WARNING]
>
> This does mean that even if you omit a title in the body of your Markdown document it is
> impossible not to have a title shown anyway at the moment, as Nuxt will automatically
> generate a title from any of the `# Title Markup`, `title: metadata` or `filepath.md`.

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
