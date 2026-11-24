import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Start here',
      collapsed: false,
      items: ['index', 'getting-started', 'status', 'roadmap', 'limitations'],
    },
    {
      type: 'category',
      label: 'Concepts',
      collapsed: false,
      items: ['concepts/the-loop', 'concepts/search-engine', 'concepts/spaces', 'concepts/studio'],
    },
    {
      type: 'category',
      label: 'Guides',
      collapsed: false,
      items: ['guides/run-a-round'],
    },
    {
      type: 'category',
      label: 'Reference',
      collapsed: false,
      items: ['reference/skill', 'reference/metrics', 'reference/glossary', 'reference/specification'],
    },
  ],
};

export default sidebars;
