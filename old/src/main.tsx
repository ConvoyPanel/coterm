import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'
import { MantineProvider, createEmotionCache } from '@mantine/core'

const emotionCache = createEmotionCache({
    key: 'mantine',
    prepend: false,
})

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <MantineProvider
            emotionCache={emotionCache}
            theme={{
                fontFamily: `Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";`,
            }}
        >
            <App />
        </MantineProvider>
    </React.StrictMode>
)
