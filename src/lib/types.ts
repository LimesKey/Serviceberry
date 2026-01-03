export type WifiEntry = {
	ssid: string | null;
	bssid: string; // MAC address as string
	age?: number; // ms, optional
	channel_info: {
		frequency_mhz: number;
		channel: number | null;
		bandwidth_mhz?: number;
	};
	radioType: string[]; // array of PHY type strings
	signalStrength: number;
	wifi_security: {
		version: string | null;
		group_cipher?: string | null;
		pairwise_ciphers?: string[];
		auth_suites?: string[];
		mfp?: string | null;
	};
	capabilities?: string[];
};

export type BtEntry = {
	name: string | null;
	address: string;
	rssi: number | null;
	uuids: string[];
	lastSeen?: number; // timestamp in ms
};

export type LogItem = {
	id: string;
	ts: string;
	level: "info" | "warn" | "error" | "debug";
	msg: string;
};

export type AppSettings = {
	logLevel: "info" | "warn" | "error" | "debug";
	testMode: boolean;
	showVisualization: boolean;
	visualizationBand: "2.4GHz" | "5GHz";
};
