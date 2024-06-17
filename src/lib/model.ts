export interface LiveInfo {
	url: string;
	anchorName: string;
	anchorAvatar: string;
	title: string;
	status: LiveStatus;
	viewerCount: string;
	roomCover: string;
	streams: Array<Stream>;
	platformKind: PlatformKind;
}

export enum PlatformKind {
	Douyin = 'Douyin',
	Tiktok = 'Tiktok',
	Xiaohongshu = 'Xiaohongshu',
	Huya = 'Huya',
	Douyu = 'Douyu',
	Kuaishou = 'Kuaishou',
	Bilibili = 'Bilibili',
	Twitch = 'Twitch',
	Youtube = 'Youtube',
	Unknown = 'Unknown'
}

export interface Stream {
	url: string;
	resolution: string;
	protocol: StreamingProtocol;
}

export enum StreamingProtocol {
	Flv = 'Flv',
	Hls = 'Hls'
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

export interface RecordingOption {
	useProxy: String | null | undefined;
}

export interface RecordingHistory {
	url: string;
	status: RecordingStatus;
	startTime: number;
	endTime: number;
	path: string;
	fileSize: number;
	deleted: boolean;
	liveInfo?: LiveInfo;
}

export type RecordingStrategy =
	| { kind: 'Timed'; value: [number, number] }
	| { kind: 'TimedWithDuration'; value: [number, number] }
	| { kind: 'TimedUntilAnchorEnd'; value: [number] }
	| { kind: 'AnchorLive' }
	| { kind: 'AnchorLiveWithDuration'; value: [number] };

export interface RecordingPlan {
	url: string;
	streamProtocol: string;
	streamResolution: string;
	enabled: boolean;
	createdAt: number;
	updatedAt: number;
	liveInfo?: LiveInfo;
	strategy: 'AnchorLive';
}

export interface AppConfig {
	ffmpegPath: string;
	savePath: string;
	liveInfoCheckInterval: number;
}

export interface Stream {
	url: string;
	resolution: string;
	protocol: StreamingProtocol;
}

export interface ProxyConfig {
	enabled: boolean;
	address: string;
}

export interface QueryHistory {
	url: string;
	anchorName: string;
	platformKind: PlatformKind;
	createdAt: number;
}
