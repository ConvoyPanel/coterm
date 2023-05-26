import NoTokenFoundModal from '@/components/NoTokenFoundModal'
import UnsupportedConsoleModal from '@/components/UnsupportedConsoleModal'
import { useState } from 'react'
import { z } from 'zod'

const consoleTypeSchema = z.enum(['novnc', 'xtermjs'])

const App = () => {
    const [alreadyRanMiddleware, setAlreadyRanMiddleware] = useState(false)
    const [hasErrors, setHasErrors] = useState(false)
    const [consoleType, setConsoleType] = useState<z.infer<
        typeof consoleTypeSchema
    > | null>(null)
    const port = import.meta.env.DEV ? 3000 : undefined

    if (!alreadyRanMiddleware || hasErrors) {
        const queryParams = new URLSearchParams(window.location.search)
        const queryConsoleType = consoleTypeSchema.safeParse(
            queryParams.get('type')
        )
        const token = queryParams.get('token')

        if (queryConsoleType.success) {
            setConsoleType(queryConsoleType.data)
        } else {
            if (!hasErrors) setHasErrors(true)

            return <NoTokenFoundModal open={true} />
        }

        if (!token || token.length === 0) {
            if (!hasErrors) setHasErrors(true)

            return <UnsupportedConsoleModal open={true} />
        }

        document.cookie = `token=${token}; max-age=30; path=/`
        setAlreadyRanMiddleware(true)
    }

    return (
        <>
            {consoleType === 'novnc' && (
                <iframe
                    src={`noVNC/vnc.html?${
                        port ? `port=${port}&` : ''
                    }path=ws&resize=scale&autoconnect=1&reconnect=1&reconnect_delay=500`}
                    className='w-full h-full'
                ></iframe>
            )}
            {consoleType === 'xtermjs' && (
                <p>Xterm.js is unsupported at the moment</p>
            )}
        </>
    )
}

export default App
