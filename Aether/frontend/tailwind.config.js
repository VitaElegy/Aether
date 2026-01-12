/** @type {import('tailwindcss').Config} */
export default {
    darkMode: 'class',
    content: [
        "./index.html",
        "./src/**/*.{vue,js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            fontFamily: {
                sans: ['Inter', 'system-ui', 'sans-serif'],
                serif: ['"Cormorant Garamond"', '"Playfair Display"', 'Georgia', 'serif'],
                display: ['"Pinyon Script"', 'cursive'],
                mono: ['"JetBrains Mono"', 'monospace'],
            },
            colors: {
                paper: 'rgb(var(--color-paper) / <alpha-value>)',
                ink: 'rgb(var(--color-ink) / <alpha-value>)',
                ash: 'rgb(var(--color-ash) / <alpha-value>)',
                accent: 'rgb(var(--color-accent) / <alpha-value>)',
            },
            spacing: {
                '128': '32rem',
            },
            typography: ({ theme }) => ({
                DEFAULT: {
                    css: {
                        '--tw-prose-body': 'rgb(var(--color-ink))',
                        '--tw-prose-headings': 'rgb(var(--color-ink))',
                        '--tw-prose-lead': 'rgb(var(--color-ink))',
                        '--tw-prose-links': 'rgb(var(--color-ink))',
                        '--tw-prose-bold': 'rgb(var(--color-ink))',
                        '--tw-prose-counters': 'rgb(var(--color-ink))',
                        '--tw-prose-bullets': 'rgb(var(--color-ink))',
                        '--tw-prose-hr': 'rgb(var(--color-ash))',
                        '--tw-prose-quotes': 'rgb(var(--color-ink))',
                        '--tw-prose-quote-borders': 'rgb(var(--color-ash))',
                        '--tw-prose-captions': 'rgb(var(--color-ink) / 0.6)',
                        '--tw-prose-code': 'rgb(var(--color-ink))',
                        '--tw-prose-pre-code': 'rgb(var(--color-ink))',
                        '--tw-prose-pre-bg': 'rgb(var(--color-ash))',
                        '--tw-prose-th-borders': 'rgb(var(--color-ash))',
                        '--tw-prose-td-borders': 'rgb(var(--color-ash))',
                    },
                },
            }),
        },
    },
    plugins: [
        require('@tailwindcss/typography'),
    ],
}
