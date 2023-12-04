<script lang='ts'>
    import * as Card from '$lib/components/ui/card'
    import { Button } from '$lib/components/ui/button'
    import { page } from '$app/stores'
</script>

<div class='w-full h-full grid place-items-center p-3'>
    <Card.Root class='max-w-lg w-full'>
        <Card.Header>
            <Card.Title>XTerm.js is unsupported</Card.Title>
        </Card.Header>
        <Card.Content>
            <p>We're sorry. Convoy terminal currently doesn't have support for proxying XTerm.js sessions.</p>

            <h4 class='tracking-tight font-semibold text-md pt-4'>If you're a programmer</h4>
            Help is needed to reverse engineer the network packets sent by XTerm.js on Proxmox. If you're interested in
            helping, please checkout the <a href='https://github.com/ConvoyPanel/coterm' target='_blank'
                                            class='underline underline-offset-4'>repository</a> or reach out in the <a
            href='https://discord.convoypanel.com' target='_blank'
            class='underline underline-offset-4'>Discord community</a>.
        </Card.Content>
    </Card.Root>
</div>