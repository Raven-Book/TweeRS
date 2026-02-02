import * as path from 'node:path';
import { defineConfig } from 'rspress/config';

export default defineConfig({
  root: path.join(__dirname, 'docs'),
  title: 'TweeRS',
  base: '/TweeRS/',
  themeConfig: {
    socialLinks: [
      {
        icon: 'github',
        mode: 'link',
        content: 'https://github.com/Raven-Book/TweeRS',
      },
    ],
  },
  builderConfig: {
    resolve: {
      alias: {
        '@': path.join(__dirname, 'src'),
      },
    },
    tools: {
      rspack: {
        experiments: {
          asyncWebAssembly: true,
        },
      },
    },
  },
});
