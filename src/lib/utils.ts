// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function debounce<F extends (...args: any[]) => any>(
	fn: F,
	delay: number
): (...args: Parameters<F>) => void {
	let timeoutId: ReturnType<typeof setTimeout> | null = null;

	return (...args: Parameters<F>): void => {
		if (timeoutId) {
			clearTimeout(timeoutId);
		}

		timeoutId = setTimeout(() => {
			fn(...args);
		}, delay);
	};
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function throttle<F extends (...args: any[]) => any>(
	fn: F,
	delay: number
): (...args: Parameters<F>) => void {
	let lastCall = 0;

	return (...args: Parameters<F>): void => {
		const now = new Date().getTime();
		if (now - lastCall < delay) {
			return;
		}
		lastCall = now;
		fn(...args);
	};
}

// 获取分辨率的中文表示
export function getResolutionName(resolution: string) {
	switch (resolution.toLowerCase()) {
		case 'full_hd1':
			return '原画';
		case 'hd1':
			return '超清';
		case 'sd1':
			return '高清';
		case 'sd2':
			return '标清';
		case 'default':
			return '默认';
		default:
			return resolution;
	}
}

// 格式化显示文件尺寸
export function formatFileSize(bytes: number) {
	if (bytes === 0) return '0 B';

	const k = 1024;
	const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));

	return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

export function closeDialog(id: string) {
	let dialog = document.getElementById(id) as HTMLDialogElement;
	dialog?.close();
}

export function openDialog(id: string) {
	let dialog = document.getElementById(id) as HTMLDialogElement;
	dialog?.showModal();
}
