import daisyui from 'daisyui';
import type { Config } from 'tailwindcss';
import { addDynamicIconSelectors } from '@iconify/tailwind';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {
			colors: {
				dark: '#212121',
				black1: '#171717',
				gray1: '#2F2F2F',
				gray2: '#676767',
				white1: '#ECECEC',
				white2: '#A2A3A7'
			},
			maxWidth: {
				'2/3': '66.666667%'
			}
		}
	},
	plugins: [daisyui, addDynamicIconSelectors()],
	daisyui: {
		themes: ['dark']
	}
} as Config;
