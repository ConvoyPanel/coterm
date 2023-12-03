<script lang='ts'>
    import type { PageData } from './$types'
    import NoVNC from '$lib/components/terminals/NoVNC.svelte'
    import XTermJS from '$lib/components/terminals/XTermJS.svelte'

    export let data: PageData

    const terminals = {
        'novnc': NoVNC,
        'xtermjs': XTermJS,
    }
</script>

<svelte:component this={terminals[data.type]} />
