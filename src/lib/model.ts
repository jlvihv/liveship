export interface LiveInfo {
	url: string;
	anchorName: string;
	anchorAvatar: string;
	title: string;
	status: LiveStatus;
	viewerCount: string;
	roomCover: string;
	streams: Array<Stream>;
	platformKind: string;
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
	id: number;
	url: string;
	streamProtocol: string;
	streamResolution: string;
	enabled: boolean;
	createdAt: number;
	updatedAt: number;
	liveInfo?: LiveInfo;
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
