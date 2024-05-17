// pub struct LiveInfo {
//     // 主播 id
//     pub anchor_id: String,
//     // 主播名
//     pub anchor_name: String,
//     // 主播头像
//     pub anchor_avatar: String,
//     // 直播间标题
//     pub title: String,
//     // 直播状态
//     pub status: LiveStatus,
//     // 多少人正在看
//     pub viewer_count: String,
// }

export interface ApiResponse {
	code: number;
	message: string;
	data: unknown;
}

export interface LiveInfo {
	anchorName: string;
	anchorAvatar: string;
	title: string;
	status: LiveStatus;
	viewerCount: string;
	roomCover: string;
	streamUrl: {
		defaultResolution: string;
		flv: Array<[string, string]>;
		hls: Array<[string, string]>;
	};
}

export enum LiveStatus {
	Live = 'Live',
	NotLive = 'NotLive'
}

export interface StorageSetting {
	path: string;
	filename: string;
}

export enum RecordingStatus {
	Recording = 'Recording',
	NotRecording = 'NotRecording'
}

export interface RecordingHistory {
	id: number;
	url: string;
	status: RecordingStatus;
	startTime: number;
	endTime: number;
	path: string;
	fileSize: number;
	deleted: boolean;
	liveRoomInfo: LiveInfo;
}

export type RecordingStrategy =
	| { kind: 'Timed'; value: [number, number] }
	| { kind: 'TimedWithDuration'; value: [number, number] }
	| { kind: 'TimedUntilAnchorEnd'; value: [number] }
	| { kind: 'AnchorLive' }
	| { kind: 'AnchorLiveWithDuration'; value: [number] };

export interface RecordingPlan {
	id: number;
	url: string;
	streamKind: string;
	streamResolution: string;
	enabled: boolean;
	createdAt: number;
	updatedAt: number;
	liveRoomInfo?: LiveInfo;
}

export interface AppConfig {
	ffmpegPath: string;
	savePath: string;
	liveInfoCheckInterval: number;
}
