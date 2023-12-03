<script lang='ts'>
    import { getBackendHost, getBackendPort } from '$lib/api/http'

    const url = new URL('/noVNC/vnc.html', `${window.location.protocol}//${window.location.host}`)

    const params = {
        port: getBackendPort().toString(),
        host: getBackendHost(),
        path: 'ws',
        resize: 'scale',
        autoconnect: '1',
        reconnect: '1',
        reconnect_delay: '500',
    }

    for (const [key, value] of Object.entries(params)) {
        url.searchParams.append(key, value)
    }
</script>

<iframe
    title='noVNC terminal client'
    src={url.toString()}
    class='w-full h-full'
></iframe>