import type { LiveInfo } from './model';

export function createLiveInfo() {
	let liveInfo: LiveInfo | undefined = $state();
	return {
		get liveInfo() {
			return liveInfo;
		},
		set: (newLiveInfo: LiveInfo) => {
			liveInfo = newLiveInfo;
		}
	};
}
