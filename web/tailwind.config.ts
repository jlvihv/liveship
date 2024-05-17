import daisyui from 'daisyui';
import { primary, secondary } from 'daisyui/src/theming';
/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {}
	},

	plugins: [daisyui],
	darkMode: ['selector', '[data-theme="dark"]'],
	daisyui: {
		themes: [
			{
				dark: {
					...require('daisyui/src/theming/themes')['dark'],
					primary: '#1e40af',
					secondary: 'teal'
				}
			},
			{
				light: {
					...require('daisyui/src/theming/themes')['light'],
					primary: '#3b82f6',
					secondary: 'teal'
				}
			}
		]
	}
};
