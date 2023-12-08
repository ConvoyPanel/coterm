<script lang='ts'>
    import { onDestroy, onMount } from 'svelte'
    import { Terminal } from 'xterm'
    import { FitAddon } from 'xterm-addon-fit'
    import { AttachAddon } from 'xterm-addon-attach'
    import 'xterm/css/xterm.css'
    import { getBackendHost, getBackendPort } from '$lib/api/http'

    let terminalDom: HTMLElement
    let handleWindowResize: () => void

    onMount(() => {
        const xterm = new Terminal()

        /* Load add-ons */
        const fitAddon = new FitAddon()
        xterm.loadAddon(fitAddon)

        /* Attach to websocket */
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
        const socket = new WebSocket(`${protocol}//${getBackendHost()}:${getBackendPort()}/ws`)
        socket.binaryType = 'arraybuffer'
        socket.onmessage = (event) => {

            const response = new Uint8Array(event.data)
            xterm.write(response)
        }

        xterm.onData((data) => {
            socket.send('0:' + unescape(encodeURIComponent(data)).length.toString() + ':' + data)
        })

        xterm.open(terminalDom)

        fitAddon.fit()
        handleWindowResize = () => {
            fitAddon.fit()
        }

        window.addEventListener('resize', handleWindowResize)
    })

    onDestroy(() => {
        window.removeEventListener('resize', handleWindowResize)
    })
</script>

<div bind:this={terminalDom} class='terminal h-full' />