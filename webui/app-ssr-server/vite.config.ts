import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig(({ command, mode }) => {
  const isServer = mode === 'server';

  return {
    plugins: [tailwindcss(), solidPlugin()],
    build: {
      outDir: isServer ? 'dist' : 'dist/public',
      rollupOptions: {
        input: isServer ? 'src/server.tsx' : 'src/client.tsx',
        output: isServer ? {
          entryFileNames: 'server.js',
          format: 'es'
        } : {
          entryFileNames: 'client.js',
          chunkFileNames: 'client-[hash].js',
          assetFileNames: 'client-[hash].[ext]'
        }
      },
      ssr: isServer,
      minify: false // Keep readable for debugging in development
    },
    server: {
      port: 5173,
      hmr: {
        port: 5174
      }
    }
  };
});
