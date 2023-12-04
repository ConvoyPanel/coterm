<script lang='ts'>
    import { page } from '$app/stores'
    import * as Card from '$lib/components/ui/card'
    import { Button } from '$lib/components/ui/button'
</script>

<div class='w-full h-full grid place-items-center p-3'>
    <Card.Root class='max-w-lg w-full'>
        <Card.Header>
            <Card.Title>Error {$page.status} | {$page.error.message}</Card.Title>
        </Card.Header>
        <Card.Content>
            <p>We're sorry. Convoy terminal encountered an error when initializing your console.</p>
            <h4 class='tracking-tight font-semibold text-md pt-4'>If you're a client</h4>
            <p>Verify that you opened Convoy terminal through your server's dashboard. Also, sessions cannot be
                resumed
                after they're disconnected, so a new session must be created. Other than that, incessantly bug your
                administrator if
                this is a reoccurring issue.</p>
            <h4 class='tracking-tight font-semibold text-md pt-4'>If you're a system administrator</h4>
            <p>Double check your deployment of <a class='underline underline-offset-4'
                                                  href='https://github.com/ConvoyPanel/coterm/releases' target='_blank'>Coterm
                is up-to-date</a> (your version: <strong class='font-medium'>{import.meta.env.APP_VERSION}</strong>)
                and that the
                version you have is
                supported
                with your version of Convoy. Additionally,
                verify that your configuration for Convoy terminal is correct. <br /><br /> We're not sure how helpful
                these
                tips
                are, but they're worth a try. </p>
        </Card.Content>
        {#if import.meta.env.CONVOY_URL}
            <Card.Footer class='flex justify-end'>
                <Button href={import.meta.env.CONVOY_URL}>Return to Convoy</Button>
            </Card.Footer>
        {/if}
    </Card.Root>
</div>