import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Prova',
  description: 'Verifiable storage anchored to Ethereum.',
  cleanUrls: true,
  lastUpdated: true,
  ignoreDeadLinks: true,

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/prova-mark.svg' }],
    ['meta', { name: 'theme-color', content: '#0a0c0f' }],
  ],

  themeConfig: {
    siteTitle: 'Prova',
    logo: '/prova-mark.svg',

    nav: [
      { text: 'Home', link: '/' },
      { text: 'Quickstart', link: '/getting-started/quickstart' },
      { text: 'API', link: '/api/' },
      { text: 'CLI', link: '/cli/auth' },
      { text: 'SDK', link: '/sdk/' },
      { text: 'GitHub', link: 'https://github.com/prova-network' },
    ],

    sidebar: [
      {
        text: 'Getting started',
        items: [
          { text: 'Web upload',  link: '/getting-started/web-upload' },
          { text: 'CLI',         link: '/getting-started/cli' },
          { text: 'Quickstart',  link: '/getting-started/quickstart' },
        ],
      },
      {
        text: 'Concepts',
        items: [
          { text: 'How Prova works',           link: '/concepts/architecture' },
          { text: 'Piece-CIDs',                link: '/concepts/piece-cids' },
          { text: 'Deal lifecycle',            link: '/concepts/deal-lifecycle' },
          { text: 'Continuous proof',          link: '/concepts/continuous-proof' },
          { text: 'Resilience',                link: '/concepts/resilience' },
        ],
      },
      {
        text: 'API reference',
        items: [
          { text: 'Overview',                    link: '/api/' },
          { text: 'Authentication',              link: '/api/authentication' },
          { text: 'POST /api/auth/signup',       link: '/api/auth-signup' },
          { text: 'POST /api/upload',            link: '/api/upload' },
          { text: 'GET /p/{cid}',                link: '/api/retrieve' },
          { text: 'GET /api/files',              link: '/api/files' },
          { text: 'GET /api/usage',              link: '/api/usage' },
          { text: 'GET /api/tokens/list',        link: '/api/tokens-list' },
          { text: 'POST /api/tokens/revoke',     link: '/api/tokens-revoke' },
        ],
      },
      {
        text: 'CLI reference',
        items: [
          { text: 'prova auth',    link: '/cli/auth' },
          { text: 'prova put',     link: '/cli/put' },
          { text: 'prova get',     link: '/cli/get' },
          { text: 'prova ls',      link: '/cli/ls' },
          { text: 'prova whoami',  link: '/cli/whoami' },
          { text: 'prova logout',  link: '/cli/logout' },
        ],
      },
      {
        text: 'SDK',
        items: [
          { text: 'Overview', link: '/sdk/' },
          { text: 'Storage',  link: '/sdk/storage' },
          { text: 'Payments', link: '/sdk/payments' },
        ],
      },
      {
        text: 'Run a node',
        items: [
          { text: 'Become a prover',     link: '/provers/become-a-prover' },
          { text: 'Hardware',            link: '/provers/hardware' },
          { text: 'Earnings',            link: '/provers/earnings' },
          { text: 'Hobby (laptop / NAS)', link: '/provers/hobby' },
          { text: 'Prosumer (home rack)', link: '/provers/prosumer' },
          { text: 'Enterprise',          link: '/provers/enterprise' },
        ],
      },
      {
        text: 'Reference',
        items: [
          { text: 'Glossary',  link: '/reference/glossary' },
          { text: 'Errors',    link: '/reference/errors' },
          { text: 'Changelog', link: '/reference/changelog' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/prova-network' },
    ],

    search: {
      provider: 'local',
    },

    footer: {
      message: 'Apache-2.0 OR MIT.',
      copyright: '© 2026 Prova',
    },

    editLink: {
      pattern: 'https://github.com/prova-network/docs/edit/main/:path',
      text: 'Edit this page on GitHub',
    },
  },
})
