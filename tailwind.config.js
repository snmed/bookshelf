/** @type {import('tailwindcss').Config} */
import daisyui from 'daisyui';
import tailwindcssTypography from '@tailwindcss/typography';
import daisyThemes from 'daisyui/src/theming/themes';

/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/app.html', './src/**/*.{js,ts,jsx,tsx,svelte}'],
  theme: {
    extend: {},
  },
  plugins: [tailwindcssTypography, daisyui],

  daisyui: {
    themes: [
      {
        light: {
          ...daisyThemes['[data-theme=fantasy]'],
          '--rounded-btn': '0.3rem',
        },
      },
      {
        dark: {
          ...daisyThemes['[data-theme=dracula]'],
          '--rounded-btn': '0.3rem',
        },
      },
    ],
  },
};
