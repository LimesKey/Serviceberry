<script lang="ts">
import { fly } from "svelte/transition";
import type { BtEntry } from "../lib/types";
import { writable } from "svelte/store";
import { getRelativeTime } from "../lib/utils";
import { onMount, onDestroy } from "svelte";

export let items: BtEntry[] = [];
export let lastScan: string | null = null;

const filter = writable("");
const sortKey = writable<"rssi" | "name" | "lastSeen">("rssi");
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
	.map((b, i) => ({ ...b, _key: b.address ?? `${i}` })) // ensure unique key
	.filter((b) => {
		const q = $filter.trim().toLowerCase();
		if (!q) return true;
		const name = (b.name ?? "Unknown").toLowerCase();
		const addr = (b.address ?? "").toLowerCase();
		return name.includes(q) || addr.includes(q);
	})
	.sort((a, b) => {
		if ($sortKey === "rssi") {
			return $sortDir * ((b.rssi ?? -999) - (a.rssi ?? -999));
		} else if ($sortKey === "lastSeen") {
			const diff = $sortDir * ((b.lastSeen ?? 0) - (a.lastSeen ?? 0));
			if (diff !== 0) return diff;
		} else {
			const an = (a.name ?? "Unknown").toLowerCase();
			const bn = (b.name ?? "Unknown").toLowerCase();
			const diff = $sortDir * an.localeCompare(bn);
			if (diff !== 0) return diff;
		}
		return a._key.localeCompare(b._key); // stable
	});

function toggleSort(key: "rssi" | "name" | "lastSeen") {
	if ($sortKey === key) sortDir.set($sortDir === 1 ? -1 : 1);
	else sortKey.set(key);
}

function rssiColor(rssi: number | null) {
	if (rssi === null) return "#9CA3AF";
	if (rssi > -50) return "#10b981";
	if (rssi > -70) return "#f59e0b";
	return "#ef4444";
}

function rssiPercent(rssi: number): number {
	return Math.min(100, Math.max(0, ((rssi + 100) / 70) * 100));
}
</script>

<section class="panel bt-panel">
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
				<path d="M6.5 6.5l11 11m-11 0l11-11M12 2v20" />
			</svg>
			<div>
				<h3>Bluetooth Devices</h3>
				<div class="count">
					{filtered.length}
					{filtered.length === 1 ? "device" : "devices"}
				</div>
			</div>
		</div>
		<div class="header-right">
			<input
				type="search"
				placeholder="Filter name or address..."
				bind:value={$filter}
				aria-label="Filter Bluetooth devices"
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
						on:click={() => toggleSort("name")}
						on:keypress={(e) => e.key === "Enter" && toggleSort("name")}
						tabindex="0"
						role="button"
					>
						Device Name
						{#if $sortKey === "name"}
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
					<th>Address</th>
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
						><td colspan="4" class="empty-state">
							<svg
								width="48"
								height="48"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.5"
							>
								<path d="M6.5 6.5l11 11m-11 0l11-11M12 2v20" />
							</svg>
							<p>No Bluetooth devices found</p>
							<span>Try adjusting your filter or scanning again</span>
						</td></tr
					>
				{:else}
					{#each filtered as b (b._key)}
						<tr>
							<td>
								<div class="device-name">
									<strong>{b.name ?? "Unknown Device"}</strong>
								</div>
							</td>
							<td><code class="mono">{b.address ?? "—"}</code></td>
							<td>
								<div class="signal-cell">
									{#if b.rssi !== null}
										<div
											class="signal-bar"
											role="progressbar"
											aria-valuenow={rssiPercent(b.rssi)}
											aria-valuemin="0"
											aria-valuemax="100"
										>
											<div
												class="signal-fill"
												style="width: {rssiPercent(
													b.rssi
												)}%; background: {rssiColor(b.rssi)}"
											></div>
										</div>
										<span class="signal-value">{b.rssi} dBm</span>
									{:else}
										<span class="no-signal">No data</span>
									{/if}
								</div>
							</td>
							<td>
								<span class="last-seen">
									{#if b.lastSeen}
										{#key now}
											{getRelativeTime(b.lastSeen)}
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
	max-height: 450px; /* Reduced from 600px */
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
	font-size: var(--text-sm); /* Increased for readability */
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

.device-name strong {
	color: var(--color-text);
	font-size: var(--text-base);
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
}

.signal-fill {
	height: 100%;
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

.no-signal {
	color: var(--color-text-muted);
	font-style: italic;
	font-size: var(--text-sm);
}

.last-seen {
	font-size: var(--text-sm);
	color: var(--color-text-secondary);
	font-family: var(--font-mono);
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
