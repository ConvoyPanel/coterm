import { useEffect } from 'react'

const App = () => {
    const queryParams = new URLSearchParams(window.location.search)
    const consoleType = queryParams.get('consoleType')
    const token = queryParams.get('token')

    useEffect(() => {
        document.cookie = `token=${token}; path=/`
    }, [])


    return (
        <>
            <iframe src="noVNC/vnc.html?host=localhost&port=3000&path=ws&resize=scale&autoconnect=1&reconnect=1&reconnect_delay=500" className='w-full h-full'></iframe>
        </>
    )
}

export default App
