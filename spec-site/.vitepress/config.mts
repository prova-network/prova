import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Prova Spec',
  description: 'Formal specifications for the Prova storage protocol on Base.',
  cleanUrls: true,
  lastUpdated: true,
  ignoreDeadLinks: true,

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/prova-mark.svg' }],
    ['meta', { name: 'theme-color', content: '#0F4C5C' }],
    ['meta', { property: 'og:title', content: 'Prova Spec' }],
    ['meta', { property: 'og:description', content: 'Formal specifications for the Prova storage protocol on Base.' }],
  ],

  themeConfig: {
    siteTitle: 'Prova Spec',
    logo: '/prova-mark.svg',

    nav: [
      { text: 'Spec home',  link: '/' },
      { text: 'Status',     link: '/status' },
      { text: 'Whitepaper', link: 'https://prova.network/whitepaper' },
      { text: 'Docs',       link: 'https://docs.prova.network' },
      { text: 'GitHub',     link: 'https://github.com/prova-network/prova/tree/main/spec' },
    ],

    sidebar: [
      {
        text: '1. Introduction',
        items: [
          { text: '1.1 Spec home',          link: '/' },
          { text: '1.2 Status overview',    link: '/status' },
          { text: '1.3 Conventions',        link: '/conventions' },
        ],
      },
      {
        text: '2. Storage proofs',
        items: [
          { text: '2.1 PDP integration',    link: '/pdp-integration' },
          { text: '2.2 Checkpoint anchoring', link: '/checkpoint-anchoring' },
          { text: '2.3 Data availability',  link: '/data-availability' },
        ],
      },
      {
        text: '3. Deal lifecycle',
        items: [
          { text: '3.1 Marketplace',        link: '/marketplace' },
          { text: '3.2 Event schema',       link: '/event-schema' },
        ],
      },
      {
        text: '4. Network',
        items: [
          { text: '4.1 Network protocol',   link: '/network-protocol' },
          { text: '4.2 API gateway',        link: '/api-gateway' },
        ],
      },
      {
        text: '5. Token economics',
        items: [
          { text: '5.1 Token economics',    link: '/token-economics' },
          { text: '5.2 Governance',         link: '/governance' },
        ],
      },
      {
        text: '6. Security',
        items: [
          { text: '6.1 Threat model',       link: '/security-threat-model' },
          { text: '6.2 Audit checklist',    link: '/security-audit-checklist' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/prova-network/prova/tree/main/spec' },
    ],

    footer: {
      message: 'Apache-2.0 OR MIT.',
      copyright: '© 2026 Prova',
    },

    search: {
      provider: 'local',
    },
  },
})
