import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.slam.web',
  appName: 'SlamWeb',
  webDir: 'dist',
  server: {
    androidScheme: 'http',
    cleartext: true,
  },
  plugins: {
    CapacitorHttp: {
      enabled: true,
    },
    CapacitorCookies: {
      enabled: true,
    },
  },
};

export default config;
