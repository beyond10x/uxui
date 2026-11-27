import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import {withProductSite} from '@beyond10x/docs-system/product-site';

const config: Config = {
  title: 'uxui',
  tagline: 'A measured improvement loop for web UIs, and the specification of a UX search engine.',
  favicon: 'img/favicon.svg',

  future: {
    v4: true,
  },

  url: 'https://beyond10x.github.io',
  baseUrl: '/uxui/',

  organizationName: 'beyond10x',
  projectName: 'uxui',
  deploymentBranch: 'gh-pages',
  trailingSlash: false,

  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',

  markdown: {
    format: 'detect',
    mermaid: true,
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  themes: ['@docusaurus/theme-mermaid'],

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: 'https://github.com/beyond10x/uxui/tree/main/website/',
        },
        blog: false,
        theme: {
          customCss: ['./src/css/landing.css', './src/css/site.css'],
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    navbar: {
      title: 'uxui',
      items: [
        {to: '/docs/', label: 'Docs', position: 'left'},
        {to: '/docs/getting-started', label: 'Get started', position: 'left'},
        {to: '/docs/guides/run-a-round', label: 'Guides', position: 'left'},
        {to: '/docs/reference/skill', label: 'Reference', position: 'left'},
        {to: '/docs/status', label: 'Status', position: 'left'},
        {href: 'https://github.com/beyond10x/uxui', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'Overview', to: '/docs/'},
            {label: 'Install the plugin', to: '/docs/getting-started'},
            {label: 'Status', to: '/docs/status'},
            {label: 'Roadmap', to: '/docs/roadmap'},
            {label: 'Limitations', to: '/docs/limitations'},
          ],
        },
        {
          title: 'Concepts',
          items: [
            {label: 'The loop', to: '/docs/concepts/the-loop'},
            {label: 'The search engine', to: '/docs/concepts/search-engine'},
            {label: 'Design and state spaces', to: '/docs/concepts/spaces'},
            {label: 'The studio', to: '/docs/concepts/studio'},
          ],
        },
        {
          title: 'Reference',
          items: [
            {label: 'The ui-iteration-loop skill', to: '/docs/reference/skill'},
            {label: 'Metric catalogue', to: '/docs/reference/metrics'},
            {label: 'Glossary', to: '/docs/reference/glossary'},
            {label: 'ESS specification', to: '/docs/reference/specification'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'Source', href: 'https://github.com/beyond10x/uxui'},
            {label: 'Changelog', href: 'https://github.com/beyond10x/uxui/blob/main/CHANGELOG.md'},
          ],
        },
      ],
      copyright: 'A beyond10x project. uxui · Apache-2.0.',
    },
  } satisfies Preset.ThemeConfig,
};

export default withProductSite(config, {landing: './product.json', mark: 'Ux'});
