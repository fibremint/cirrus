
import path from 'path';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite'
import Unocss from 'unocss/vite'
import { internalIpV4 } from 'internal-ip'


const SRC_DIR = path.resolve(__dirname, './src');
const PUBLIC_DIR = path.resolve(__dirname, './public');
const BUILD_DIR = path.resolve(__dirname, './dist',);

// export default defineConfig(async ({ command, mode }) => {
//   const host = process.env.TAURI_PLATFORM === 'android' || process.env.TAURI_PLATFORM === 'ios' ? (await internalIpV4()) : 'localhost'

//   return {
//     plugins: [Unocss(), svelte()],
//     build: {
//       rollupOptions: {
//         output: {
//           entryFileNames: `assets/[name].js`,
//           chunkFileNames: `assets/[name].js`,
//           assetFileNames: `assets/[name].[ext]`
//         }
//       }
//     },
//     server: {
//       host: '0.0.0.0',
//       port: 5173,
//       strictPort: true,
//       hmr: {
//         protocol: 'ws',
//         host,
//         port: 5183
//       },
//       fs: {
//         allow: ['.', '../../tooling/api/dist']
//       }
//     }
//   }
// })

const host = process.env.TAURI_PLATFORM === 'android' || process.env.TAURI_PLATFORM === 'ios' ? (await internalIpV4()) : 'localhost'

export default {
  plugins: [
    svelte(),

  ],
  root: SRC_DIR,
  base: '',
  publicDir: PUBLIC_DIR,
  build: {
    outDir: BUILD_DIR,
    assetsInlineLimit: 0,
    emptyOutDir: true,
    rollupOptions: {
      treeshake: false,
    },
  },
  resolve: {
    alias: {
      '@': SRC_DIR,
    },
  },
  server: {
    host: '0.0.0.0',
    port: 5173,
    strictPort: true,
    hmr: {
      protocol: 'ws',
      host,
      port: 5183
    },
    fs: {
      allow: ['.', '../../tooling/api/dist']
    }
  }
};
