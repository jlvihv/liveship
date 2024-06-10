// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://beta.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;
import { loadTranslations } from '$lib/translations';

export const load = async ({ url }: { url: any }) => {
	const { pathname } = url;

	const initLocale = localStorage.getItem('lang') || 'en';

	await loadTranslations(initLocale, pathname);

	// 动态导入 checkPlanLoop
	const { checkPlanLoop } = await import('$lib/backstage');
	checkPlanLoop();

	return {};
};
