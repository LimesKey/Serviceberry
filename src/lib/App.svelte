<script lang="ts">
import { onMount, onDestroy } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import ConfigPanel from "../components/ConfigPanel.svelte";
import WifiTable from "../components/WifiTable.svelte";
import BtTable from "../components/BtTable.svelte";
import Logs from "../components/Logs.svelte";
import NetworkVisualization from "../components/NetworkVisualization.svelte";
import type { WifiEntry, BtEntry, LogItem } from "./types";
import { formatTimeWithoutSeconds } from "./utils";
import "../app.css";

let wifiResults: WifiEntry[] = [];
let btResults: BtEntry[] = [];
let logs: LogItem[] = [];

let scanIntervalMs = 50000;
let autoScan = true;
let adapter = "wlan0";
let dwellTime = 200;
let endpoint = "https://example.com/geosubmit";
let testMode = false;
let logLevel: "info" | "warn" | "error" | "debug" = "info";
let showVisualization = true;
let visualizationExpanded = true;
let visualizationBand: "2.4GHz" | "5GHz" = "2.4GHz";

let adapterOptions: string[] = [];
let settingsOpen = false;

let _scanTimer: number | null = null;
let lastWifiScan: string | null = null;
let lastBtScan: string | null = null;
let lastError: string | null = null;

let wifiScanning = false;
let btScanning = false;
$: isScanning = wifiScanning || btScanning;

function makeId() {
	try {
		return crypto.randomUUID();
	} catch {
		return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
	}
}

function pushLog(msg: string, level: LogItem["level"] = "info") {
	// Filter logs based on log level
	const levels = ["debug", "info", "warn", "error"];
	const currentLevelIndex = levels.indexOf(logLevel);
	const msgLevelIndex = levels.indexOf(level);

	if (msgLevelIndex < currentLevelIndex) return;

	const item: LogItem = {
		id: makeId(),
		ts: formatTimeWithoutSeconds(new Date()),
		level,
		msg,
	};
	logs = [item, ...logs].slice(0, 500);
}

function formatSecurity(security: any): string {
	if (!security) return "Open";
	if (typeof security === "string") return security; // fallback

	const version = security.version || "WPA";
	const authSuite = security.auth_suites?.[0];
	const cipher = security.pairwise_ciphers?.[0];

	if (!authSuite) return version;
	if (authSuite === "PSK") return `${version}-PSK`;
	if (authSuite === "SAE") return `${version}-SAE`;
	if (authSuite === "EAP") return `${version}-EAP`;
	return `${version}-${authSuite}`;
}

function normalizeWifi(res: any[]): WifiEntry[] {
	const now = Date.now();
	const incoming = (res ?? []).map((w) => {
		// Map backend fields to WifiEntry
		const ssid = (w.ssid ?? "").trim() || null;
		const bssid = w.macAddress ?? w.bssid ?? "unknown";
		const rssi = typeof w.signalStrength === "number" ? w.signalStrength : (w.rssi ?? -999);
		const channel = w.channel ?? null;
		const channel_info = w.channel_info ?? null;
		const radioType = w.radioType ?? w.radio_type ?? null;
		const signalStrength = typeof w.signalStrength === "number" ? w.signalStrength : (w.rssi ?? -999);
		const wifi_security = w.wifi_security ?? w.security ?? null;
		const security = formatSecurity(w.security ?? w.wifi_security);
		const lastSeen = now;
		const seen = Array.isArray(w.seen) ? w.seen : [];
		const vendor = w.vendor ?? null;
		const country = w.country ?? null;
		const information_elements = w.information_elements ?? [];
		const capabilities = w.capabilities ?? null;
		const frequency = w.frequency ?? null;
		const width = w.width ?? null;
		const center_freq1 = w.center_freq1 ?? null;
		const center_freq2 = w.center_freq2 ?? null;
		const beacon_interval = w.beacon_interval ?? null;
		const tsf = w.tsf ?? null;
		const ie = w.ie ?? null;
		const raw = w.raw ?? null;
		return {
			ssid,
			bssid,
			rssi,
			channel,
			channel_info,
			radioType,
			signalStrength,
			wifi_security,
			security,
			lastSeen,
			seen,
			vendor,
			country,
			information_elements,
			capabilities,
			frequency,
			width,
			center_freq1,
			center_freq2,
			beacon_interval,
			tsf,
			ie,
			raw,
		};
	});
	const map = new Map(wifiResults.map((w) => [w.bssid, w]));
	for (const w of incoming) map.set(w.bssid, w);
	return Array.from(map.values()).sort((a, b) => a.bssid.localeCompare(b.bssid));
}

function normalizeBt(res: any[]): BtEntry[] {
	const now = Date.now();
	const incoming = (res ?? []).map((b) => ({
		name: b.name ?? "Unknown",
		address: b.address,
		rssi: typeof b.rssi === "number" ? b.rssi : null,
		uuids: Array.isArray(b.uuids) ? b.uuids : [],
		lastSeen: now,
	}));
	const map = new Map(btResults.map((b) => [b.address, b]));
	for (const b of incoming) map.set(b.address, b);
	return Array.from(map.values()).sort((a, b) =>
		a.address.localeCompare(b.address)
	);
}

async function fetchWifi() {
	if (!testMode) {
		pushLog("Wi-Fi scan requires iOS position data (enable test mode)", "warn");
		return;
	}

	wifiScanning = true;
	pushLog("Starting Wi-Fi scan", "debug");
	try {
		const res = (await invoke("list_wifi")) as any[];
		wifiResults = normalizeWifi(res);
		lastWifiScan = formatTimeWithoutSeconds(new Date());
		pushLog(`Wi-Fi scan updated - ${wifiResults.length} networks found`);
	} catch (e) {
		lastError = String(e);
		pushLog(`Wi-Fi scan error: ${String(e)}`, "error");
	} finally {
		wifiScanning = false;
	}
}

async function fetchBt() {
	btScanning = true;
	pushLog("Starting Bluetooth scan", "debug");
	try {
		const res = (await invoke("list_bt")) as any[];
		btResults = normalizeBt(res);
		lastBtScan = formatTimeWithoutSeconds(new Date());
		pushLog(`Bluetooth scan updated - ${btResults.length} devices found`);
	} catch (e) {
		lastError = String(e);
		pushLog(`Bluetooth scan error: ${String(e)}`, "error");
	} finally {
		btScanning = false;
	}
}

async function loadAdapters() {
	try {
		const res = (await invoke("list_adapters")) as string[];
		adapterOptions = res;
		if (!adapterOptions.includes(adapter) && adapterOptions.length > 0) {
			adapter = adapterOptions[0];
		}
	} catch (e) {
		pushLog(`Adapter list error: ${String(e)}`, "warn");
	}
}

async function runScans() {
	await Promise.allSettled([fetchWifi(), fetchBt()]);
}

function startPeriodicScanning() {
	stopPeriodicScanning();
	if (!autoScan) return;
	_scanTimer = window.setInterval(runScans, scanIntervalMs);
}

function stopPeriodicScanning() {
	if (_scanTimer !== null) {
		clearInterval(_scanTimer);
		_scanTimer = null;
	}
}

function handleConfigApply(e: CustomEvent) {
	const {
		scanIntervalSeconds: s,
		adapter: a,
		dwellTime: d,
		endpoint: ep,
		logLevel: ll,
		visualizationBand: vb,
	} = e.detail;
	scanIntervalMs = Number(s) * 1000;
	adapter = a;
	dwellTime = Number(d);
	endpoint = ep;
	logLevel = ll || "info";
	visualizationBand = vb || "2.4GHz";
	pushLog("Configuration updated", "info");
	startPeriodicScanning();
}

function handleToggleAuto(e: CustomEvent) {
	autoScan = e.detail.auto;
	pushLog(`Automatic scanning ${autoScan ? "enabled" : "disabled"}`, "info");
	if (autoScan) startPeriodicScanning();
	else stopPeriodicScanning();
}

function handleToggleTestMode(e: CustomEvent) {
	testMode = e.detail.testMode;
	pushLog(`Test mode ${testMode ? "enabled" : "disabled"}`, "info");
}

function handleToggleVisualization(e: CustomEvent) {
	showVisualization = e.detail.showVisualization;
	if (!showVisualization) visualizationExpanded = false;
	pushLog(
		`Network visualization ${showVisualization ? "shown" : "hidden"}`,
		"info"
	);
}

async function handleSubmitGeo() {
	pushLog("Submitting geolocation…");
	try {
		await invoke("submit_geo", { position: null, cell_towers: null });
		pushLog("Geosubmit successful");
	} catch (e) {
		pushLog(`Geosubmit error: ${String(e)}`, "error");
	}
}

onMount(() => {
	pushLog("Serviceberry started", "info");
	loadAdapters();
	if (testMode) {
		runScans();
	}
	startPeriodicScanning();
});

onDestroy(() => {
	stopPeriodicScanning();
});
</script>

<main class="app">
	<header class="topbar">
		<div class="brand">
			<svg
				width="32"
				height="32"
				viewBox="0 0 32 32"
				aria-hidden="true"
				class="logo"
			>
				<circle cx="16" cy="16" r="14" fill="url(#gradient)" />
				<defs>
					<linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="100%">
						<stop offset="0%" style="stop-color:#3b82f6" />
						<stop offset="100%" style="stop-color:#8b5cf6" />
					</linearGradient>
				</defs>
			</svg>
			<div>
				<h1>Serviceberry</h1>
				<div class="subtitle">Wi‑Fi & Bluetooth Scanner</div>
			</div>
		</div>

		<div class="status-bar">
			<div class="status-indicator">
				<span class="status-dot" class:scanning={isScanning}></span>
				<span class="status-text">{isScanning ? "Scanning…" : "Ready"}</span>
			</div>
			<button
				class="btn-icon"
				on:click={() => runScans()}
				aria-label="Refresh scans"
				title="Refresh scans"
			>
				<svg
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"
					/>
				</svg>
			</button>
		</div>
	</header>

	<div class="container">
		<div class="sidebar">
			<div class="sidebar-header">
				<button
					class="btn-toggle"
					on:click={() => (settingsOpen = !settingsOpen)}
					aria-expanded={settingsOpen}
				>
					<svg
						width="18"
						height="18"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<circle cx="12" cy="12" r="3" />
						<path
							d="M12 1v6m0 6v6M5.6 5.6l4.2 4.2m4.2 4.2l4.2 4.2M1 12h6m6 0h6M5.6 18.4l4.2-4.2m4.2-4.2l4.2-4.2"
						/>
					</svg>
					{settingsOpen ? "Hide Settings" : "Settings"}
				</button>
			</div>

			{#if settingsOpen}
				<div class="settings-panel">
					<ConfigPanel
						scanIntervalMs={scanIntervalMs}
						autoScan={autoScan}
						adapter={adapter}
						adapterOptions={adapterOptions}
						dwellTime={dwellTime}
						endpoint={endpoint}
						testMode={testMode}
						logLevel={logLevel}
						showVisualization={showVisualization}
						visualizationBand={visualizationBand}
						on:apply={handleConfigApply}
						on:toggleAuto={handleToggleAuto}
						on:toggleTestMode={handleToggleTestMode}
						on:toggleVisualization={handleToggleVisualization}
						on:submitGeo={handleSubmitGeo}
					/>
				</div>
			{/if}

			<Logs items={logs} />
		</div>

		<main class="main-content">
			<div class="scan-grid">
				<WifiTable items={wifiResults} lastScan={lastWifiScan} />

				<div class="visualization-wrapper">
					<button
						class="viz-toggle"
						on:click={() => (visualizationExpanded = !visualizationExpanded)}
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<polyline
								points={visualizationExpanded
									? "6 9 12 15 18 9"
									: "9 6 15 12 9 18"}
							></polyline>
						</svg>
						<span>Network Visualization</span>
					</button>
					{#if visualizationExpanded}
						<div class="band-selector">
							<button
								class:active={visualizationBand === "2.4GHz"}
								on:click={() => (visualizationBand = "2.4GHz")}>2.4 GHz</button
							>
							<button
								class:active={visualizationBand === "5GHz"}
								on:click={() => (visualizationBand = "5GHz")}>5 GHz</button
							>
						</div>
						<NetworkVisualization
							networks={wifiResults}
							band={visualizationBand}
						/>
					{/if}
				</div>

				<BtTable items={btResults} lastScan={lastBtScan} />
			</div>
		</main>
	</div>
</main>

<style>
.app {
	min-height: 100vh;
	display: flex;
	flex-direction: column;
	background: var(--color-bg);
}

.topbar {
	background: var(--color-surface);
	border-bottom: 1px solid var(--color-border);
	padding: var(--space-sm) var(--space-lg);
	display: flex;
	justify-content: space-between;
	align-items: center;
	gap: var(--space-md);
	flex-wrap: wrap;
	box-shadow: var(--shadow-sm);
	position: sticky;
	top: 0;
	z-index: var(--z-sticky);
}

.brand {
	display: flex;
	align-items: center;
	gap: var(--space-sm);
}

.logo {
	flex-shrink: 0;
	width: 28px;
	height: 28px;
}

.brand h1 {
	font-size: var(--text-xl);
	font-weight: var(--font-bold);
	color: var(--color-text);
	margin: 0;
}

.subtitle {
	font-size: var(--text-xs);
	color: var(--color-text-muted);
	margin-top: 1px;
}

.status-bar {
	display: flex;
	align-items: center;
	gap: var(--space-md);
}

.status-indicator {
	display: flex;
	align-items: center;
	gap: var(--space-sm);
	padding: var(--space-sm) var(--space-md);
	background: var(--color-bg);
	border-radius: var(--radius-full);
	border: 1px solid var(--color-border);
}

.status-dot {
	width: 8px;
	height: 8px;
	border-radius: 50%;
	background: var(--color-success);
	transition: all var(--transition-base);
}

.status-dot.scanning {
	background: var(--color-warning);
	animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
	0%,
	100% {
		opacity: 1;
		transform: scale(1);
	}
	50% {
		opacity: 0.5;
		transform: scale(1.1);
	}
}

.status-text {
	font-size: var(--text-sm);
	font-weight: var(--font-medium);
	color: var(--color-text-secondary);
}

.btn-icon {
	background: var(--color-bg);
	border: 1px solid var(--color-border);
	padding: var(--space-sm);
	border-radius: var(--radius-md);
	color: var(--color-text-secondary);
	transition: all var(--transition-fast);
}

.btn-icon:hover {
	background: var(--color-border);
	color: var(--color-text);
	transform: rotate(180deg);
}

.container {
	flex: 1;
	display: grid;
	grid-template-columns: 280px 1fr;
	gap: 0;
	overflow: hidden;
}

.sidebar {
	background: var(--color-surface);
	border-right: 1px solid var(--color-border);
	display: flex;
	flex-direction: column;
	overflow: hidden;
}

.sidebar-header {
	padding: var(--space-sm) var(--space-md);
	border-bottom: 1px solid var(--color-border);
}

.btn-toggle {
	width: 100%;
	background: var(--color-primary);
	color: white;
	padding: var(--space-sm) var(--space-md);
	font-weight: var(--font-medium);
	border-radius: var(--radius-md);
	transition: background var(--transition-fast);
}

.btn-toggle:hover {
	background: var(--color-primary-hover);
}

.settings-panel {
	border-bottom: 1px solid var(--color-border);
}

.main-content {
	overflow: auto;
	padding: var(--space-md) var(--space-lg);
}

.scan-grid {
	display: grid;
	gap: var(--space-md);
	grid-template-columns: 1fr;
}

.visualization-wrapper {
	background: var(--color-surface);
	border-radius: var(--radius-lg);
	border: 1px solid var(--color-border);
	overflow: hidden;
	box-shadow: var(--shadow-sm);
}

.viz-toggle {
	display: flex;
	align-items: center;
	gap: var(--space-sm);
	width: 100%;
	padding: var(--space-md) var(--space-lg);
	border: none;
	border-bottom: 1px solid var(--color-border);
	background: var(--color-bg);
	color: var(--color-text);
	font-weight: var(--font-semibold);
	cursor: pointer;
	transition: background var(--transition-fast);
}

.viz-toggle:hover {
	background: var(--color-border);
}

.band-selector {
	display: flex;
	gap: var(--space-xs);
	padding: var(--space-md) var(--space-lg) 0;
	border-bottom: 1px solid var(--color-border);
}

.band-selector button {
	padding: var(--space-sm) var(--space-md);
	border: 1px solid var(--color-border);
	border-radius: var(--radius-sm);
	background: transparent;
	color: var(--color-text-secondary);
	font-size: var(--text-sm);
	cursor: pointer;
	transition: all var(--transition-fast);
}

.band-selector button.active {
	background: var(--color-primary);
	color: white;
	border-color: var(--color-primary);
}

@media (max-width: 1024px) {
	.container {
		grid-template-columns: 1fr;
	}

	.sidebar {
		border-right: none;
		border-bottom: 1px solid var(--color-border);
		max-height: 50vh;
	}
}

@media (max-width: 768px) {
	.topbar {
		padding: var(--space-md);
	}

	.brand h1 {
		font-size: var(--text-xl);
	}

	.main-content {
		padding: var(--space-md);
	}

	.scan-grid {
		gap: var(--space-md);
	}
}

@media (max-width: 480px) {
	.status-bar {
		width: 100%;
		justify-content: space-between;
	}
}
</style>
