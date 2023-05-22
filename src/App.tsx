import { useEffect, useRef, useState } from 'react'
import { Terminal } from 'xterm'
import { AttachAddon } from 'xterm-addon-attach'
import { FitAddon } from 'xterm-addon-fit'

const App = () => {
    const terminalElement = useRef<HTMLDivElement>(null)
    const [running, setRunning] = useState(false)

    useEffect(() => {
        if (running) return
        setRunning(true)
        const terminal = new Terminal()
        terminal.loadAddon(new FitAddon())

        const socket = new WebSocket('ws://localhost:3000/ws')
        terminal.loadAddon(new AttachAddon(socket))

        terminal.open(terminalElement.current!)

        return () => {
            terminal.dispose()
        }
    }, [])

    return (
        <>
            <div id='terminal' ref={terminalElement} className='w-full h-full'></div>
            {/* <iframe src="noVNC/vnc.html?host=localhost&port=3000&path=ws&resize=scale&autoconnect=1&reconnect=1&reconnect_delay=500" className='w-full h-full'></iframe> */}
        </>
    )
}

export default App
