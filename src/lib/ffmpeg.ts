import { invoke } from '@tauri-apps/api/core';

export interface MediaInfo {
	path: string;
	mediaType: 'image' | 'video';
	width: number;
	height: number;
	fps?: number;
	bitrate?: number;
}

export interface Effects {
	snow?: boolean;
	music?: string; // path to music file
}

export function generateFFmpegCommand(mediaInfo: MediaInfo, pushUrl: string): string[] {
	const { path, mediaType: type, width, height, fps, bitrate } = mediaInfo;
	const defaultBitrate = 3000;
	const defaultFps = 30;
	const finalBitrate = bitrate || defaultBitrate;
	const finalFps = fps || defaultFps;
	const bufsize = 2 * finalBitrate;

	// Validate inputs
	if (!path || !type || !width || !height) {
		throw new Error('Invalid input parameters.');
	}

	// Adjust width and height to maintain aspect ratio
	const aspectRatio = width / height;
	const targetWidth = Math.round(width / 2) * 2; // Ensure width is even
	const targetHeight = Math.round(targetWidth / aspectRatio / 2) * 2; // Ensure height is even

	// Use appropriate scaling filter and format
	const scaleFilter = `scale=${targetWidth}:${targetHeight}:flags=lanczos,format=yuv420p`;

	let ffmpegCommand = '';

	if (type === 'image') {
		ffmpegCommand =
			`ffmpeg -loop 1 -i ${path} -c:v libx264 -preset ultrafast -tune zerolatency ` +
			`-pix_fmt yuv420p -s ${targetWidth}x${targetHeight} -b:v ${finalBitrate}k -maxrate ${finalBitrate}k ` +
			`-bufsize ${bufsize}k -vf ${scaleFilter} -r ${finalFps} -f flv ${pushUrl}`;
	} else if (type === 'video') {
		ffmpegCommand =
			`ffmpeg -i ${path} -c:v libx264 -preset ultrafast -tune zerolatency ` +
			`-pix_fmt yuv420p -s ${targetWidth}x${targetHeight} -b:v ${finalBitrate}k -maxrate ${finalBitrate}k ` +
			`-bufsize ${bufsize}k -vf ${scaleFilter} -r ${finalFps} -f flv ${pushUrl}`;
	} else {
		throw new Error('Unsupported media type.');
	}
	// 以空格分割，去掉空字符串，去掉第一个元素，返回数组
	return ffmpegCommand.split(' ').filter(Boolean).slice(1);
}

function parseFFmpegOutput(output: string, filePath: string): MediaInfo {
	const lines = output.split('\n');
	let type: 'image' | 'video' = 'video';
	let width = 0;
	let height = 0;
	let fps: number | undefined;
	let bitrate: number | undefined;

	for (const line of lines) {
		if (line.includes('Video:')) {
			const videoMatch = line.match(/Video: ([^,]+)/);
			if (videoMatch && videoMatch[1].includes('mjpeg')) {
				type = 'image';
			}

			const dimensionMatch = line.match(/, (\d+)x(\d+)/);
			if (dimensionMatch) {
				width = parseInt(dimensionMatch[1], 10);
				height = parseInt(dimensionMatch[2], 10);
			}

			const fpsMatch = line.match(/, (\d+(?:\.\d+)?) fps/);
			if (fpsMatch) {
				fps = parseFloat(fpsMatch[1]);
			}
		}

		if (line.includes('bitrate:')) {
			const bitrateMatch = line.match(/bitrate: (\d+) kb\/s/);
			if (bitrateMatch) {
				bitrate = parseInt(bitrateMatch[1], 10);
			}
		}
	}

	return { path: filePath, mediaType: type, width, height, fps, bitrate };
}

export async function getImageInfo(filePath: string): Promise<MediaInfo> {
	try {
		const mediaInfo = await invoke('get_image_info', { filePath });
		return mediaInfo as MediaInfo;
	} catch (e) {
		console.error('getMediaInfo error', e);
		throw new Error('Failed to get media info.');
	}
}

export function buildFFmpegForwardCommand(url: string, outputUrl: string): string[] {
	const userAgent = `"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"`;
	const analyzeduration = '20000000';
	const probesize = '10000000';
	const bufsize = '8000k';
	const maxMuxingQueueSize = '1024';

	const ffmpegCommand = [
		'-y',
		'-v',
		'verbose',
		'-rw_timeout',
		'30000000',
		'-loglevel',
		'error',
		'-hide_banner',
		'-user_agent',
		userAgent,
		'-protocol_whitelist',
		'rtmp,crypto,file,http,https,tcp,tls,udp,rtp',
		'-thread_queue_size',
		'1024',
		'-analyzeduration',
		analyzeduration,
		'-probesize',
		probesize,
		'-fflags',
		'+discardcorrupt',
		'-i',
		url,
		'-bufsize',
		bufsize,
		'-sn',
		'-dn',
		'-reconnect_delay_max',
		'60',
		'-reconnect_streamed',
		'1',
		'-reconnect_at_eof',
		'1',
		'-max_muxing_queue_size',
		maxMuxingQueueSize,
		'-correct_ts_overflow',
		'1',
		'-c:v',
		'copy',
		'-c:a',
		'copy',
		'-f',
		'flv',
		outputUrl,
		'-vf',
		"setpts='PTS+0.1*RANDOM(1)'"
	];

	return ffmpegCommand;
}
