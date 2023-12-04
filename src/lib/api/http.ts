import axios, { type AxiosInstance } from 'axios'

export const getBackendPort = (): number => {
    const BACKEND_URL = import.meta.env.BACKEND_URL
    if (
        BACKEND_URL !== undefined &&
        BACKEND_URL !== null &&
        BACKEND_URL !== ''
    ) {
        const url = new URL(BACKEND_URL)

        if (url.port !== '') {
            return parseInt(url.port)
        }
    }

    if (import.meta.env.DEV) {
        return 3000
    }

    if (window.location.port !== '') {
        return parseInt(window.location.port)
    }

    return window.location.protocol === 'https:' ? 443 : 80
}

export const getBackendHost = () => {
    const BACKEND_URL = import.meta.env.BACKEND_URL
    if (
        BACKEND_URL !== undefined &&
        BACKEND_URL !== null &&
        BACKEND_URL !== ''
    ) {
        const url = new URL(BACKEND_URL)

        if (url.hostname !== '') {
            return url.hostname
        }
    }

    if (import.meta.env.DEV) {
        return 'localhost'
    }

    return window.location.hostname
}

export const getBaseUrl = () => {
    const BACKEND_URL = import.meta.env.BACKEND_URL
    if (
        BACKEND_URL !== undefined &&
        BACKEND_URL !== null &&
        BACKEND_URL !== ''
    ) {
        return import.meta.env.BACKEND_URL
    }

    if (import.meta.env.DEV) {
        return 'http://localhost:3000'
    }

    return '/'
}

const http: AxiosInstance = axios.create({
    baseURL: getBaseUrl(),
    headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
    },
})

export default http
