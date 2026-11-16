/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{svelte,js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        plasma: {
          bg: '#060913',
          panel: '#0b0f19',
          card: '#0e1526',
          border: 'rgba(0, 242, 254, 0.15)',
          borderSubtle: 'rgba(255, 255, 255, 0.08)',
          cyan: '#00f2fe',
          mint: '#00f5d4',
          purple: '#9d4edd',
          violet: '#c77dff',
          pink: '#f72585',
          amber: '#ffb703',
          emerald: '#10b981',
          rose: '#ef4444',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
        sans: ['Inter', 'system-ui', 'sans-serif'],
      },
      boxShadow: {
        'plasma-cyan': '0 0 20px -5px rgba(0, 242, 254, 0.3)',
        'plasma-purple': '0 0 20px -5px rgba(157, 78, 221, 0.3)',
        'plasma-mint': '0 0 20px -5px rgba(0, 245, 212, 0.3)',
        'plasma-glow': '0 0 35px -5px rgba(0, 242, 254, 0.25), 0 0 20px -5px rgba(157, 78, 221, 0.2)',
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'spin-slow': 'spin 12s linear infinite',
        'spin-reverse': 'spin-reverse 16s linear infinite',
      },
      keyframes: {
        'spin-reverse': {
          to: { transform: 'rotate(-360deg)' },
        },
      },
    },
  },
  plugins: [],
};
