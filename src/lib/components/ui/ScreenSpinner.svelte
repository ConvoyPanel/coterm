<script lang='ts'>
    import type { HTMLAttributes } from 'svelte/elements'
    import { clsx } from 'clsx'
    import LogoSpinner from '$lib/components/ui/LogoSpinner.svelte'

    interface $$props extends HTMLAttributes<HTMLDivElement> {
    }

    let className = ''
    export { className as class }
</script>

<div class={clsx('grid', 'place-items-center', 'w-full', 'h-full', className)}>
    <LogoSpinner />
</div>