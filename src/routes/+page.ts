import type { PageLoad } from './$types'
import { z } from 'zod'
import { error } from '@sveltejs/kit'

const consoleTypeSchema = z.enum(['novnc', 'xtermjs'])

const consoleSessionSchema = z.object({
    type: consoleTypeSchema,
    token: z.string().min(1),
})

export const load: PageLoad = ({ params, url }) => {
    const searchParams = url.searchParams
    const validated = consoleSessionSchema.safeParse({
        type: searchParams.get('type'),
        token: searchParams.get('token'),
    })

    if (!validated.success) {
        throw error(400, 'Invalid Console Session')
    }

    const session = validated.data

    document.cookie = `token=${session.token}; max-age=30; path=/`

    return session
}
