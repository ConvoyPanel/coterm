<script lang='ts'>
    import { onDestroy, onMount } from 'svelte'
    import { Terminal } from 'xterm'
    import { FitAddon } from 'xterm-addon-fit'
    import 'xterm/css/xterm.css'
    import { getBackendHost, getBackendPort } from '$lib/api/http'
    import ScreenSpinner from '$lib/components/ui/ScreenSpinner.svelte'
    import { toast } from 'svelte-sonner'
    import { fade } from 'svelte/transition'

    let terminalDom: HTMLElement

    let xterm: Terminal
    let fitAddon: FitAddon
    let isConnected = false

    let idlePingInterval: ReturnType<typeof setInterval>

    onMount(() => {
        xterm = new Terminal()

        /* Load add-ons */
        fitAddon = new FitAddon()
        xterm.loadAddon(fitAddon)

        /* Attach to websocket */
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
        const socket = new WebSocket(`${protocol}//${getBackendHost()}:${getBackendPort()}/ws`)
        socket.binaryType = 'arraybuffer'

        socket.addEventListener('message', (e: MessageEvent) => {
            const res = new Uint8Array(e.data)
            if (!isConnected && res.length === 2 && res[0] === 79 && res[1] === 75) {
                isConnected = true
                toast.success('Connected to console')

                return
            }
            xterm.write(res)
        })

        xterm.onResize((dimensions) => {
            if (!isConnected) return

            socket.send('1:' + dimensions.cols.toString() + ':' + dimensions.rows.toString() + ':')
        })

        xterm.onData((data) => {
            socket.send('0:' + data.length.toString() + ':' + data)
        })

        idlePingInterval = setInterval(() => {
            if (!isConnected) return

            socket.send('2')
        }, 30_000)

        xterm.open(terminalDom)

        fitAddon.fit()
    })

    onDestroy(() => {
        clearInterval(idlePingInterval)
    })
</script>

<svelte:window on:resize={() => fitAddon.fit()} />

{#if !isConnected}
    <div transition:fade={{duration:150}} class='bg-background absolute inset-0 z-10'>
        <ScreenSpinner />
    </div>
{/if}

<div bind:this={terminalDom} class='terminal h-full' />