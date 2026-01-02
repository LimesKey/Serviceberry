<script lang="ts">
import type { WifiEntry } from "../lib/types";
import { getChannelBand } from "../lib/utils";

export let networks: WifiEntry[] = [];
export let band: "2.4GHz" | "5GHz" = "2.4GHz";

// filter networks by band and sort by signal strength
$: filteredNetworks = networks
	.filter((n) => n.channel && getChannelBand(n.channel) === band)
	.sort((a, b) => (b.rssi ?? -100) - (a.rssi ?? -100));

// cahart dimensions
const width = 800;
const height = 340;
const padding = { top: 40, right: 30, bottom: 55, left: 60 };
const chartWidth = width - padding.left - padding.right;
const chartHeight = height - padding.top - padding.bottom;

// RSSI range
const maxRssi = -30;
const minRssi = -90;

// Y-axis ticks
const yTicks = [-30, -40, -50, -60, -70, -80, -90];

// X-axis channel configuration
$: channelConfig =
	band === "2.4GHz"
		? {
				min: 1,
				max: 14,
				pad: 2.0,
				ticks: Array.from({ length: 14 }, (_, i) => 1 + i),
				labelStep: 1,
			}
		: {
				min: 32,
				max: 177,
				pad: 5.0,
				ticks: Array.from({ length: 146 }, (_, i) => 32 + i),
				labelStep: 10,
			};

let effectiveMin: number;
let effectiveMax: number;

// Dynamically zoom 5GHz to the active cluster of channels
$: {
	const baseMin = channelConfig.min;
	const baseMax = channelConfig.max;
	if (band === "5GHz" && filteredNetworks.length > 0) {
		const channels = filteredNetworks
			.map((n) => n.channel)
			.filter((c): c is number => typeof c === "number");
		if (channels.length) {
			const dataMin = Math.min(...channels);
			const dataMax = Math.max(...channels);
			const margin = 12;
			const minSpan = 60;
			let spanMin = Math.max(baseMin, dataMin - margin);
			let spanMax = Math.min(baseMax, dataMax + margin);
			if (spanMax - spanMin < minSpan) {
				const extra = (minSpan - (spanMax - spanMin)) / 2;
				spanMin = Math.max(baseMin, spanMin - extra);
				spanMax = Math.min(baseMax, spanMax + extra);
			}
			effectiveMin = spanMin;
			effectiveMax = spanMax;
		} else {
			effectiveMin = baseMin;
			effectiveMax = baseMax;
		}
	} else {
		effectiveMin = baseMin;
		effectiveMax = baseMax;
	}
}

// Padded domain to keep edge channels off the frame boundary
$: paddedMin = effectiveMin - channelConfig.pad;
$: paddedMax = effectiveMax + channelConfig.pad;

// Scale functions
function scaleX(channel: number): number {
	const clamped = Math.min(Math.max(channel, paddedMin), paddedMax);
	return (
		padding.left +
		((clamped - paddedMin) / (paddedMax - paddedMin)) * chartWidth
	);
}

function scaleY(rssi: number): number {
	const clamped = Math.min(Math.max(rssi, minRssi), maxRssi);
	return (
		padding.top +
		chartHeight -
		((clamped - minRssi) / (maxRssi - minRssi)) * chartHeight
	);
}

// Build lookup of networks per channel for offset calculation
function computeNetworkOffsets(networks: WifiEntry[]): Map<string, number> {
	const channelGroups: Record<number, WifiEntry[]> = {};
	networks.forEach((n) => {
		if (!n.channel) return;
		channelGroups[n.channel] = channelGroups[n.channel] || [];
		channelGroups[n.channel].push(n);
	});

	const offsets = new Map<string, number>();
	Object.entries(channelGroups).forEach(([ch, nets]) => {
		nets.forEach((n, i) => {
			// Offset labels vertically to avoid overlap
			offsets.set(n.bssid, i * 18);
		});
	});
	return offsets;
}

$: networkOffsets = computeNetworkOffsets(filteredNetworks);

// Generate curve path for a network
function generateCurvePath(network: WifiEntry): string {
	if (!network.channel || !network.rssi) return "";

	const centerChannel = network.channel;
	const peakRssi = network.rssi;

	// bandwithf determines curve width (in channels)
	const bwMHz = network.bandwidth || (band === "2.4GHz" ? 20 : 40);
	const channelWidth = band === "2.4GHz" ? bwMHz / 5 : bwMHz / 10;
	const sigma = Math.max(
		0.5,
		band === "5GHz" ? (channelWidth / 2) * 1.1 : (channelWidth / 2) * 0.65
	);

	const points: [number, number][] = [];
	const range = sigma * 3.2;
	const steps = 64;
	const start = Math.max(paddedMin, centerChannel - range);
	const end = Math.min(paddedMax, centerChannel + range);

	for (let i = 0; i <= steps; i++) {
		const ch = start + ((end - start) * i) / steps;
		const decay = Math.exp(
			-Math.pow(ch - centerChannel, 2) / (2 * sigma * sigma)
		);
		const rssiAtCh = Math.max(minRssi, peakRssi - 30 * (1 - decay));
		points.push([scaleX(ch), scaleY(rssiAtCh)]);
	}

	if (points.length === 0) return "";

	// Create closed path
	const baseline = scaleY(minRssi);
	let path = `M ${scaleX(start)} ${baseline}`;
	points.forEach(([x, y]) => {
		path += ` L ${x} ${y}`;
	});
	path += ` L ${scaleX(end)} ${baseline} Z`;

	return path;
}

// Color palette with good contrast
const colors = [
	"#3b82f6",
	"#ef4444",
	"#10b981",
	"#f59e0b",
	"#8b5cf6",
	"#ec4899",
	"#06b6d4",
	"#f97316",
	"#6366f1",
	"#84cc16",
	"#14b8a6",
	"#a855f7",
	"#f43f5e",
	"#0ea5e9",
];

function getColor(index: number): string {
	return colors[index % colors.length];
}
</script>

<section class="visualization-panel">
	<div class="chart-container">
		{#if filteredNetworks.length === 0}
			<div class="empty-state">
				<svg
					width="48"
					height="48"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path
						d="M5 12.55a11 11 0 0 1 14.08 0M1.42 9a16 16 0 0 1 21.16 0M8.53 16.11a6 6 0 0 1 6.95 0M12 20h.01"
					/>
				</svg>
				<p>No {band} networks detected</p>
				<span>Scan for networks to see visualization</span>
			</div>
		{:else}
			<svg viewBox="0 0 {width} {height}" class="chart-svg">
				<defs>
					{#each filteredNetworks as network, i (network.bssid)}
						<linearGradient id="gradient-{i}" x1="0%" y1="0%" x2="0%" y2="100%">
							<stop offset="0%" stop-color={getColor(i)} stop-opacity="0.4" />
							<stop
								offset="100%"
								stop-color={getColor(i)}
								stop-opacity="0.05"
							/>
						</linearGradient>
					{/each}
				</defs>

				<rect
					x={padding.left}
					y={padding.top}
					width={chartWidth}
					height={chartHeight}
					fill="#edf2f7"
					rx="4"
				/>

				{#each yTicks as rssi}
					<line
						x1={padding.left}
						x2={width - padding.right}
						y1={scaleY(rssi)}
						y2={scaleY(rssi)}
						stroke="#e2e8f0"
						stroke-width="1"
					/>
					<text
						x={padding.left - 10}
						y={scaleY(rssi) + 4}
						text-anchor="end"
						fill="#64748b"
						font-size="11"
						font-family="system-ui, -apple-system, sans-serif"
					>
						{rssi}
					</text>
				{/each}

				<text
					x="15"
					y={padding.top + chartHeight / 2}
					text-anchor="middle"
					fill="#475569"
					font-size="12"
					font-weight="500"
					transform="rotate(-90, 15, {padding.top + chartHeight / 2})"
				>
					Signal (dBm)
				</text>

				<!-- X-axis grid lines and channel labels -->
				{#each channelConfig.ticks as ch}
					{#if ch >= effectiveMin && ch <= effectiveMax}
						<line
							x1={scaleX(ch)}
							x2={scaleX(ch)}
							y1={padding.top}
							y2={padding.top + chartHeight}
							stroke="#e2e8f0"
							stroke-width="0.8"
							stroke-dasharray="2 4"
						/>
						{#if (ch - channelConfig.min) % channelConfig.labelStep === 0}
							<text
								x={scaleX(ch)}
								y={height - padding.bottom + 18}
								text-anchor="middle"
								fill="#475569"
								font-size="10"
								font-family="system-ui, -apple-system, sans-serif"
								font-weight="600"
							>
								{ch}
							</text>
						{/if}
					{/if}
				{/each}

				<!-- X-axis label -->
				<text
					x={padding.left + chartWidth / 2}
					y={height - 10}
					text-anchor="middle"
					fill="#475569"
					font-size="12"
					font-weight="500"
				>
					Channel
				</text>

				<!-- Network curves (rendered back to front for proper layering) -->
				{#each [...filteredNetworks].reverse() as network, idx}
					{@const i = filteredNetworks.length - 1 - idx}
					{@const color = getColor(i)}
					{#if network.channel && network.rssi}
						<path
							d={generateCurvePath(network)}
							fill="url(#gradient-{i})"
							stroke={color}
							stroke-width="2.2"
							stroke-linejoin="round"
							opacity="0.92"
						/>
					{/if}
				{/each}

				{#each filteredNetworks as network, i (network.bssid)}
					{#if network.channel && network.rssi}
						{@const labelX = Math.min(
							width - padding.right + 4,
							Math.max(padding.left - 4, scaleX(network.channel))
						)}
						{@const labelY =
							scaleY(network.rssi) -
							8 -
							(networkOffsets.get(network.bssid) || 0)}
						{@const displayName = (
							network.ssid?.trim() ||
							network.bssid ||
							"Hidden"
						).substring(0, 18)}
						{@const color = getColor(i)}
						{@const boxWidth = Math.min(150, 50 + displayName.length * 6)}

						<rect
							x={labelX - boxWidth / 2}
							y={labelY - 11}
							width={boxWidth}
							height="18"
							fill="white"
							stroke={color}
							stroke-width="1"
							rx="4"
							opacity="0.94"
						/>
						<text
							x={labelX}
							y={labelY + 2}
							text-anchor="middle"
							fill={color}
							font-size="12"
							font-weight="700"
							font-family="system-ui, -apple-system, sans-serif"
						>
							{displayName}
						</text>
					{/if}
				{/each}

				<line
					x1={padding.left}
					x2={padding.left}
					y1={padding.top}
					y2={padding.top + chartHeight}
					stroke="#94a3b8"
					stroke-width="1"
				/>
				<line
					x1={padding.left}
					x2={width - padding.right}
					y1={padding.top + chartHeight}
					y2={padding.top + chartHeight}
					stroke="#94a3b8"
					stroke-width="1"
				/>
			</svg>

			<div class="legend">
				{#each filteredNetworks.slice(0, 8) as network, i (network.bssid)}
					<div class="legend-item">
						<span class="legend-color" style="background: {getColor(i)}"></span>
						<span class="legend-text"
							>{(network.ssid?.trim() || network.bssid || "Hidden").substring(
								0,
								16
							)}</span
						>
						<span class="legend-rssi">{network.rssi} dBm</span>
					</div>
				{/each}
				{#if filteredNetworks.length > 8}
					<div class="legend-more">+{filteredNetworks.length - 8} more</div>
				{/if}
			</div>
		{/if}
	</div>
</section>

<style>
.visualization-panel {
	width: 100%;
	background: var(--color-surface);
	padding: var(--space-lg);
}

.chart-container {
	width: 100%;
	min-height: 420px;
}

.chart-svg {
	width: 100%;
	height: auto;
	display: block;
}

.legend {
	display: flex;
	flex-wrap: wrap;
	gap: var(--space-sm);
	margin-top: var(--space-md);
	padding-top: var(--space-md);
	border-top: 1px solid var(--color-border);
}

.legend-item {
	display: flex;
	align-items: center;
	gap: var(--space-xs);
	padding: 6px 10px;
	background: var(--color-bg);
	border-radius: var(--radius-sm);
	font-size: var(--text-sm);
	box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}

.legend-color {
	width: 12px;
	height: 12px;
	border-radius: 3px;
	flex-shrink: 0;
}

.legend-text {
	color: var(--color-text);
	font-weight: var(--font-semibold);
	max-width: 140px;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.legend-rssi {
	color: var(--color-text-muted);
	font-family: var(--font-mono);
	font-size: var(--text-xs);
}

.legend-more {
	display: flex;
	align-items: center;
	padding: var(--space-xs) var(--space-sm);
	color: var(--color-text-muted);
	font-size: var(--text-sm);
	font-style: italic;
}

.empty-state {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	height: 300px;
	color: var(--color-text-muted);
	gap: var(--space-md);
}

.empty-state svg {
	opacity: 0.3;
}

.empty-state p {
	font-size: var(--text-base);
	font-weight: var(--font-medium);
	margin: 0;
}

.empty-state span {
	font-size: var(--text-sm);
}

@media (max-width: 768px) {
	.legend {
		gap: var(--space-sm);
	}

	.legend-item {
		font-size: var(--text-xs);
	}

	.legend-text {
		max-width: 80px;
	}
}
</style>
