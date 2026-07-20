import { appTools, defineConfig } from '@modern-js/app-tools';

const apiProxyTarget =
  process.env.MODERN_DEV_PROXY_TARGET || 'http://127.0.0.1:3000';
const releaseDistPath = process.env.SLAM_RELEASE_DIST;
const disableSourceMap =
  process.env.SLAM_DISABLE_SOURCEMAP === 'true' || Boolean(releaseDistPath);

// https://modernjs.dev/en/configure/app/usage
export default defineConfig({
  runtime: {
    router: true,
  },
  output: {
    distPath: releaseDistPath ? { root: releaseDistPath } : undefined,
    sourceMap: disableSourceMap ? false : undefined,
    polyfill: 'off',
    disableNodePolyfill: true,
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
