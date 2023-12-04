import { sveltekit } from '@sveltejs/kit/vite'
import { defineConfig } from 'vite'
import dotenv from 'dotenv'

dotenv.config()

export default defineConfig({
    plugins: [sveltekit()],
    define: {
        'import.meta.env.APP_VERSION': JSON.stringify(
            process.env.npm_package_version
        ),
        'import.meta.env.BACKEND_URL': JSON.stringify(process.env.BACKEND_URL),
    },
})
