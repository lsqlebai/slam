import { appTools, defineConfig } from '@modern-js/app-tools';

const apiProxyTarget =
  process.env.MODERN_DEV_PROXY_TARGET || 'http://127.0.0.1:3000';

// https://modernjs.dev/en/configure/app/usage
export default defineConfig({
  runtime: {
    router: true,
  },
  plugins: [appTools({ bundler: 'rspack' })],
  dev: {
    proxy: {
      '/api': apiProxyTarget,
    },
  },
  tools: {
    devServer: {
      proxy: {
        '/api': {
          target: apiProxyTarget,
          changeOrigin: true,
          cookieDomainRewrite: 'localhost',
        },
      },
    },
  },
});
