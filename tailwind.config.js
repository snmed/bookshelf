/** @type {import('tailwindcss').Config} */
import daisyui from 'daisyui';
import daisyThemes from 'daisyui/src/theming/themes';

console.log(">>>>>>>>>>", daisyThemes);
export default {

  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,svelte}",
  ],
  theme: {
    extend: {},
  },
  plugins: [daisyui],

  daisyui: {
    themes: [ {
      light: {
        ...daisyThemes["[data-theme=fantasy]"],        
        "--rounded-btn": "0.3rem"

      }
    }, {
      dark: {
        ...daisyThemes["[data-theme=dracula]"],
        "--rounded-btn": "0.3rem"
      }
    }]
  }
}

