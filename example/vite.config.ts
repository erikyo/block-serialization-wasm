import {defineConfig} from 'vite'
import wasm from "vite-plugin-wasm";
import react from '@vitejs/plugin-react-swc'

const basePath = "/block-serialization-wasm/";

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [
        wasm(),
      react()
    ],
    base: basePath,
    server: {
        fs: {
            // Allow serving files from one level up to the project root
            allow: ['..'],
        },
    },
})
