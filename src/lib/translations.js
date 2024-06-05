import i18n from 'sveltekit-i18n';

/** @type {import('sveltekit-i18n').Config} */
const config = {
	loaders: [
		{
			locale: 'en',
			key: '',
			loader: async () => (await import('./i18n/en.json')).default
		},
		{
			locale: 'cn',
			key: '',
			loader: async () => (await import('./i18n/cn.json')).default
		}
	]
};

export const { t, locale, locales, loading, loadTranslations } = new i18n(config);
