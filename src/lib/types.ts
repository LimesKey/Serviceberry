export type WifiEntry = {
  ssid: string | null;
  bssid: string;
  rssi: number;
  channel: number | null;
  security: string; // formatted security string (WPA2-PSK, Open, etc.)
  lastSeen?: number; // timestamp in ms
  bandwidth?: number; // in MHz, optional
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
