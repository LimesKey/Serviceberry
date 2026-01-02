<script lang="ts">
import { fly } from "svelte/transition";
import type { WifiEntry } from "../lib/types";
import { writable } from "svelte/store";
import { getRelativeTime, getChannelBand } from "../lib/utils";
import { onMount, onDestroy } from "svelte";

export let items: WifiEntry[] = [];
export let lastScan: string | null = null;

const filter = writable("");
const sortKey = writable<"rssi" | "ssid" | "lastSeen">("rssi");
const sortDir = writable<1 | -1>(1);

// Live updating timestamp
let now = Date.now();
let updateInterval: number;

onMount(() => {
	updateInterval = window.setInterval(() => {
		now = Date.now();
	}, 1000);
});

onDestroy(() => {
	if (updateInterval) clearInterval(updateInterval);
});

$: filtered = items
	.filter((w) => {
		const q = $filter.trim().toLowerCase();
		if (!q) return true;
		return (
			(w.ssid ?? "").toLowerCase().includes(q) ||
			w.bssid.toLowerCase().includes(q)
		);
	})
	.sort((a, b) => {
		if ($sortKey === "rssi") {
			const diff = $sortDir * ((b.rssi ?? -999) - (a.rssi ?? -999));
			if (diff !== 0) return diff;
		} else if ($sortKey === "lastSeen") {
			const diff = $sortDir * ((b.lastSeen ?? 0) - (a.lastSeen ?? 0));
			if (diff !== 0) return diff;
		} else {
			const diff = $sortDir * (a.ssid ?? "").localeCompare(b.ssid ?? "");
			if (diff !== 0) return diff;
		}
		return a.bssid.localeCompare(b.bssid);
	});

function toggleSort(key: "rssi" | "ssid" | "lastSeen") {
	if ($sortKey === key) sortDir.set($sortDir === 1 ? -1 : 1);
	else sortKey.set(key);
}

function rssiPercent(rssi: number) {
	const p = Math.min(100, Math.max(0, Math.round(((rssi + 100) / 70) * 100)));
	return p;
}

function getSignalStrength(
	rssi: number
): "Excellent" | "Good" | "Fair" | "Weak" {
	if (rssi >= -50) return "Excellent";
	if (rssi >= -60) return "Good";
	if (rssi >= -70) return "Fair";
	return "Weak";
}

function channelBand(channel: number | null): string {
	if (channel == null) return "unknown";
	return getChannelBand(channel);
}
</script>

<section class="panel wifi-panel">
	<div class="panel-header">
		<div class="header-left">
			<svg
				width="24"
				height="24"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				class="icon"
			>
				<path
					d="M5 12.55a11 11 0 0 1 14.08 0M1.42 9a16 16 0 0 1 21.16 0M8.53 16.11a6 6 0 0 1 6.95 0M12 20h.01"
				/>
			</svg>
			<div>
				<h3>Wi-Fi Networks</h3>
				<div class="count">
					{filtered.length}
					{filtered.length === 1 ? "network" : "networks"}
				</div>
			</div>
			<div class="wifi-legend" aria-hidden="true">
				<span class="legend-item"
					><span class="legend-swatch swatch-2g"></span>2.4 GHz</span
				>
				<span class="legend-item"
					><span class="legend-swatch swatch-5g"></span>5 GHz</span
				>
			</div>
		</div>
		<div class="header-right">
			<input
				type="search"
				placeholder="Filter SSID or BSSID..."
				bind:value={$filter}
				aria-label="Filter Wi-Fi networks"
			/>
			{#if lastScan}
				<div class="timestamp" title="Last scan time">
					<svg
						width="14"
						height="14"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<circle cx="12" cy="12" r="10" />
						<polyline points="12 6 12 12 16 14" />
					</svg>
					{lastScan}
				</div>
			{/if}
		</div>
	</div>

	<div class="table-wrap" transition:fly={{ y: 6, duration: 180 }}>
		<table>
			<thead>
				<tr>
					<th
						class="sortable"
						on:click={() => toggleSort("ssid")}
						on:keypress={(e) => e.key === "Enter" && toggleSort("ssid")}
						tabindex="0"
						role="button"
					>
						SSID
						{#if $sortKey === "ssid"}
							<span
								class="sort-arrow"
								aria-label={$sortDir === 1
									? "Sorted ascending"
									: "Sorted descending"}
							>
								{$sortDir === 1 ? "↑" : "↓"}
							</span>
						{/if}
					</th>
					<th>BSSID</th>
					<th
						class="sortable"
						on:click={() => toggleSort("rssi")}
						on:keypress={(e) => e.key === "Enter" && toggleSort("rssi")}
						tabindex="0"
						role="button"
					>
						Signal
						{#if $sortKey === "rssi"}
							<span
								class="sort-arrow"
								aria-label={$sortDir === 1
									? "Sorted ascending"
									: "Sorted descending"}
							>
								{$sortDir === 1 ? "↑" : "↓"}
							</span>
						{/if}
					</th>
					<th>Ch</th>
					<th>Security</th>
					<th
						class="sortable"
						on:click={() => toggleSort("lastSeen")}
						on:keypress={(e) => e.key === "Enter" && toggleSort("lastSeen")}
						tabindex="0"
						role="button"
					>
						Last Seen
						{#if $sortKey === "lastSeen"}
							<span
								class="sort-arrow"
								aria-label={$sortDir === 1
									? "Sorted ascending"
									: "Sorted descending"}
							>
								{$sortDir === 1 ? "↑" : "↓"}
							</span>
						{/if}
					</th>
				</tr>
			</thead>
			<tbody>
				{#if filtered.length === 0}
					<tr
						><td colspan="6" class="empty-state">
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
							<p>No Wi-Fi networks found</p>
							<span>Try adjusting your filter or scanning again</span>
						</td></tr
					>
				{:else}
					{#each filtered as w (w.bssid)}
						<tr
							class:band-2g={channelBand(w.channel) === "2.4GHz"}
							class:band-5g={channelBand(w.channel) === "5GHz"}
						>
							<td class="ssid">
								<div class="ssid-cell">
									{#if (w.ssid ?? "").trim().length > 0}
										<strong class="ssid-name">{(w.ssid ?? "").trim()}</strong>
									{:else}
										<span class="hidden-ssid">(Hidden)</span>
									{/if}
								</div>
							</td>
							<td><code class="mono">{w.bssid}</code></td>
							<td>
								<div class="signal-cell">
									<div
										class="signal-bar"
										role="progressbar"
										aria-valuenow={rssiPercent(w.rssi)}
										aria-valuemin="0"
										aria-valuemax="100"
									>
										<div
											class="signal-fill"
											style="width: {rssiPercent(w.rssi)}%"
										></div>
									</div>
									<span class="signal-value">{w.rssi} dBm</span>
									<span
										class="signal-badge"
										data-strength={getSignalStrength(w.rssi)}
									>
										{getSignalStrength(w.rssi)}
									</span>
								</div>
							</td>
							<td><span class="channel-badge">{w.channel ?? "—"}</span></td>
							<td>
								<span
									class="security-badge"
									data-secure={w.security !== "Open"}
								>
									{#if w.security !== "Open"}
										<svg
											width="12"
											height="12"
											viewBox="0 0 24 24"
											fill="currentColor"
										>
											<path
												d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"
											/>
										</svg>
									{/if}
									{w.security ?? "Unknown"}
								</span>
							</td>
							<td>
								<span class="last-seen">
									{#if w.lastSeen}
										{#key now}
											{getRelativeTime(w.lastSeen)}
										{/key}
									{:else}
										—
									{/if}
								</span>
							</td>
						</tr>
					{/each}
				{/if}
			</tbody>
		</table>
	</div>
</section>

<style>
.panel {
	background: var(--color-surface);
	border-radius: var(--radius-lg);
	box-shadow: var(--shadow-sm);
	overflow: hidden;
	border: 1px solid var(--color-border);
}

.panel-header {
	display: flex;
	justify-content: space-between;
	align-items: center;
	padding: var(--space-md) var(--space-lg);
	border-bottom: 1px solid var(--color-border);
	background: linear-gradient(to bottom, var(--color-surface), var(--color-bg));
	gap: var(--space-md);
	flex-wrap: wrap;
}

.header-left {
	display: flex;
	align-items: center;
	gap: var(--space-md);
}

.icon {
	color: var(--color-primary);
}

.panel-header h3 {
	margin: 0;
	font-size: var(--text-lg);
	color: var(--color-text);
}

.count {
	font-size: var(--text-sm);
	color: var(--color-text-muted);
	margin-top: 2px;
}

.header-right {
	display: flex;
	align-items: center;
	gap: var(--space-md);
	flex: 1;
	justify-content: flex-end;
}

.header-right input {
	max-width: 280px;
	min-width: 200px;
}

.timestamp {
	display: flex;
	align-items: center;
	gap: var(--space-xs);
	font-size: var(--text-sm);
	color: var(--color-text-muted);
	white-space: nowrap;
}

.table-wrap {
	overflow: auto;
	max-height: 450px;
}

table {
	width: 100%;
	border-collapse: collapse;
}

thead {
	position: sticky;
	top: 0;
	z-index: 10;
	background: var(--color-surface);
	box-shadow: 0 1px 0 var(--color-border);
}

th {
	text-align: left;
	font-size: var(--text-sm);
	font-weight: var(--font-semibold);
	color: var(--color-text-secondary);
	padding: var(--space-md) var(--space-md);
	text-transform: uppercase;
	letter-spacing: 0.05em;
	user-select: none;
}

th.sortable {
	cursor: pointer;
	transition: background var(--transition-fast);
}

th.sortable:hover {
	background: var(--color-bg);
}

.sort-arrow {
	margin-left: var(--space-xs);
	color: var(--color-primary);
	font-size: var(--text-lg);
}

tbody tr {
	border-bottom: 1px solid var(--color-border);
	transition: background var(--transition-fast);
}

tbody tr:hover {
	background: var(--color-bg);
}

tbody tr:last-child {
	border-bottom: none;
}

td {
	padding: var(--space-md) var(--space-md);
	vertical-align: middle;
	font-size: var(--text-base);
}

.ssid-cell {
	max-width: 220px;
	display: flex;
	align-items: center;
	gap: var(--space-sm);
	justify-content: space-between;
}

.ssid-name {
	display: block;
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
	margin-right: var(--space-sm);
	min-width: 0;
	font-size: var(--text-base);
}

.ssid-cell strong {
	color: var(--color-text);
	display: block;
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

.hidden-ssid {
	color: var(--color-text-muted);
	font-style: italic;
	font-size: var(--text-xs);
}

.last-seen {
	font-size: var(--text-sm);
	color: var(--color-text-secondary);
	font-family: var(--font-mono);
}

tr.band-2g {
	background-color: rgba(6, 95, 70, 0.06);
}

tr.band-5g {
	background-color: rgba(30, 64, 175, 0.06);
}

.wifi-legend {
	display: flex;
	gap: var(--space-sm);
	margin-left: 12px;
	align-items: center;
}

.legend-item {
	display: inline-flex;
	align-items: center;
	gap: 6px;
	font-size: var(--text-xs);
	color: var(--color-text-muted);
}

.legend-swatch {
	width: 12px;
	height: 12px;
	border-radius: 3px;
	display: inline-block;
}

.swatch-2g {
	background: #065f46;
}
.swatch-5g {
	background: #1e40af;
}

.mono {
	font-family: var(--font-mono);
	font-size: var(--text-sm);
	color: var(--color-text-secondary);
	background: var(--color-bg);
	padding: 4px 8px;
	border-radius: var(--radius-sm);
	letter-spacing: 0.02em;
}

.signal-cell {
	display: flex;
	align-items: center;
	gap: var(--space-md);
}

.signal-bar {
	width: 100px;
	height: 8px;
	background: var(--color-border);
	border-radius: var(--radius-full);
	overflow: hidden;
	position: relative;
}

.signal-fill {
	height: 100%;
	background: linear-gradient(90deg, #ef4444, #f59e0b, #10b981);
	border-radius: var(--radius-full);
	transition: width var(--transition-base);
}

.signal-value {
	font-family: var(--font-mono);
	font-size: var(--text-base);
	color: var(--color-text);
	min-width: 75px;
	font-weight: var(--font-medium);
}

.signal-badge {
	font-size: var(--text-sm);
	font-weight: var(--font-medium);
	padding: 4px 10px;
	border-radius: var(--radius-sm);
	background: var(--color-border);
	color: var(--color-text-secondary);
}

.signal-badge[data-strength="Excellent"] {
	background: #d1fae5;
	color: #065f46;
}

.signal-badge[data-strength="Good"] {
	background: #dbeafe;
	color: #1e40af;
}

.signal-badge[data-strength="Fair"] {
	background: #fef3c7;
	color: #92400e;
}

.signal-badge[data-strength="Weak"] {
	background: #fee2e2;
	color: #991b1b;
}

.channel-badge {
	display: inline-block;
	padding: 6px 12px;
	background: var(--color-bg);
	border-radius: var(--radius-sm);
	font-weight: var(--font-medium);
	color: var(--color-text-secondary);
	font-size: var(--text-base);
}

.security-badge {
	display: inline-flex;
	align-items: center;
	gap: 6px;
	padding: 6px 10px;
	border-radius: var(--radius-sm);
	font-size: var(--text-sm);
	font-weight: var(--font-medium);
	background: var(--color-bg);
	color: var(--color-text-secondary);
}

.security-badge[data-secure="true"] {
	background: #d1fae5;
	color: #065f46;
}

.security-badge[data-secure="false"] {
	background: #fee2e2;
	color: #991b1b;
}

.empty-state {
	padding: var(--space-2xl) !important;
	text-align: center;
	color: var(--color-text-muted);
}

.empty-state svg {
	margin: 0 auto var(--space-md);
	opacity: 0.3;
}

.empty-state p {
	font-size: var(--text-base);
	font-weight: var(--font-medium);
	color: var(--color-text-secondary);
	margin-bottom: var(--space-xs);
}

.empty-state span {
	font-size: var(--text-sm);
}

/* Responsive */
@media (max-width: 768px) {
	.header-right input {
		min-width: 150px;
	}

	.signal-cell {
		flex-direction: column;
		align-items: flex-start;
		gap: var(--space-sm);
	}

	table {
		font-size: var(--text-xs);
	}

	th,
	td {
		padding: var(--space-sm);
	}
}
</style>
