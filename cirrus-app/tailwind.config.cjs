/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        'button-hover-light': 'rgba(0,0,0,.05)',
        'button-active-light': 'rgba(0,0,0,.1)',
        'button-hover-dark': 'hsla(0,0%,100%,.05)',
        'button-active-dark': 'hsla(0,0%,100%,.1)',
        'button-red-light': '#e81123',
        'button-red-dark': '#c42b1c',
      }
    },
  },
  plugins: [
    require("daisyui")
  ],
  // darkMode: 'class'
}
