<script>
    import '../app.pcss'
    import '@fontsource-variable/inter'
    import { toast, Toaster } from 'svelte-sonner'
    import { onMount } from 'svelte'

    onMount(() => {
        toast(`Coterm v${import.meta.env.APP_VERSION}`)
    })
</script>

<Toaster />
<slot />