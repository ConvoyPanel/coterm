<script lang='ts'>
    import type { HTMLAttributes } from 'svelte/elements'

    interface $$props extends HTMLAttributes<SVGElement> {
    }
</script>

<svg
    xmlns='http://www.w3.org/2000/svg'
    fill='none'
    stroke='currentColor'
    viewBox='0 0 512 512'
    {...$$props}
>
    <path
        stroke-linecap='round'
        d='M512,46.86c0-25.88-20.96-46.86-46.82-46.86-23.54,0-43.01,17.39-46.32,40.03l-6.94-1.49-.67-.68-.44,.44L280.03,10.25h0c-14.34-2.89-29.81,1.18-40.99,12.25-17.84,17.67-17.99,46.47-.34,64.33,6.34,6.41,14.11,10.5,22.3,12.34v.04s1.08,.23,1.08,.23c0,0,.01,0,.02,0l71.51,15.34L115,331.27c-26.57-45.91-31.16-101.03-13.84-150.11h-.03c8.73-23.94-3.31-50.33-26.88-58.94-23.58-8.62-49.77,3.8-58.5,27.73C-18.13,244.95,2.61,355.17,78.23,431.67c75.93,76.81,186.42,98.59,281.96,65.1v-.03c24.04-8.34,36.87-34.35,28.66-58.08s-34.38-36.22-58.43-27.88h0c-49.63,17.51-105.38,12.55-151.49-14.84l218.4-216.29,14.69,72.33,.26-.05c1.71,8.34,5.75,16.3,12.18,22.8,17.65,17.86,46.43,18.01,64.27,.34,12.44-12.32,16.25-30.05,11.45-45.71l-25.62-126.19,.64-.64-.97-.98-1.71-8.4c22.38-3.52,39.49-22.9,39.49-46.29Z'
    />
</svg>


<style lang='postcss'>
    svg {
        width: theme('spacing.14');
    }

    svg path {
        animation: dash 2s cubic-bezier(0.45, 0.09, 0.6, 0.89) infinite;
    }

    @keyframes dash {
        0% {
            stroke-dasharray: 100 2652;
            stroke-dashoffset: 0;
        }
        50% {
            stroke-dasharray: 2352 400;
            stroke-dashoffset: 2752;
        }
        100% {
            stroke-dasharray: 100 2652;
            stroke-dashoffset: 5504;
        }
    }

</style>