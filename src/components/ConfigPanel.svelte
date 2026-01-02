<script lang="ts">
import { createEventDispatcher, onDestroy } from "svelte";
export let scanIntervalMs: number;
export let autoScan: boolean;
export let adapter: string;
export let adapterOptions: string[] = [];
export let dwellTime: number;
export let endpoint: string;
export let testMode: boolean = false;
export let logLevel: "info" | "warn" | "error" | "debug" = "info";
export let showVisualization: boolean = false;
export let visualizationBand: "2.4GHz" | "5GHz" = "2.4GHz";

const dispatch = createEventDispatcher();

let scanIntervalSeconds = scanIntervalMs / 1000;
function toggleAuto() {
	dispatch("toggleAuto", { auto: !autoScan });
	emitApplyDebounced();
}

function toggleTestMode() {
	dispatch("toggleTestMode", { testMode: !testMode });
	emitApplyDebounced();
}

function toggleVisualization() {
	dispatch("toggleVisualization", { showVisualization: !showVisualization });
}

function submitGeo() {
	dispatch("submitGeo");
}

let _applyTimer: number | null = null;
function emitApplyDebounced() {
	if (_applyTimer !== null) clearTimeout(_applyTimer);
	_applyTimer = window.setTimeout(() => {
		dispatch("apply", {
			scanIntervalSeconds,
			adapter,
			dwellTime,
			endpoint,
			logLevel,
			visualizationBand,
		});
		_applyTimer = null;
	}, 300);
}

onDestroy(() => {
	if (_applyTimer !== null) clearTimeout(_applyTimer);
});
</script>

<section class="config-panel" aria-label="Configuration">
	<div class="config-header">
		<svg
			width="20"
			height="20"
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
		<h4>Configuration</h4>
	</div>

	<div class="config-body">
		<div class="form-group">
			<label for="scan-interval">
				<span class="label-text">Scan Interval</span>
				<span class="label-hint">How often to scan (seconds)</span>
			</label>
			<input
				id="scan-interval"
				type="number"
				min="5"
				step="1"
				bind:value={scanIntervalSeconds}
				on:input={emitApplyDebounced}
				aria-describedby="scan-interval-hint"
			/>
		</div>

		<div class="form-group">
			<label for="adapter-select">
				<span class="label-text">Network Adapter</span>
				<span class="label-hint">Wi-Fi interface to use</span>
			</label>
			<select
				id="adapter-select"
				bind:value={adapter}
				on:change={emitApplyDebounced}
			>
				{#each adapterOptions as opt}
					<option value={opt}>{opt}</option>
				{/each}
				{#if adapterOptions.length === 0}
					<option value={adapter}>No adapters detected</option>
				{/if}
			</select>
		</div>

		<div class="form-group">
			<label for="dwell-time">
				<span class="label-text">Dwell Time</span>
				<span class="label-hint">Time per channel (ms)</span>
			</label>
			<input
				id="dwell-time"
				type="number"
				min="50"
				step="50"
				bind:value={dwellTime}
				on:input={emitApplyDebounced}
			/>
		</div>

		<div class="form-group">
			<label for="endpoint">
				<span class="label-text">Submission Endpoint</span>
				<span class="label-hint">URL for geolocation data</span>
			</label>
			<input
				id="endpoint"
				type="url"
				bind:value={endpoint}
				placeholder="https://example.com/geosubmit"
				on:input={emitApplyDebounced}
			/>
		</div>

		<div class="toggle-group">
			<label class="toggle-label">
				<input
					type="checkbox"
					checked={autoScan}
					on:change={toggleAuto}
					class="toggle-input"
				/>
				<span class="toggle-switch"></span>
				<span class="toggle-text">
					<span class="toggle-title">Auto-scan</span>
					<span class="toggle-desc">Automatically scan at intervals</span>
				</span>
			</label>
		</div>

		<div class="toggle-group">
			<label class="toggle-label">
				<input
					type="checkbox"
					checked={testMode}
					on:change={toggleTestMode}
					class="toggle-input"
				/>
				<span class="toggle-switch"></span>
				<span class="toggle-text">
					<span class="toggle-title">Test Mode</span>
					<span class="toggle-desc">Scan without iOS device position data</span>
				</span>
			</label>
		</div>

		<div class="form-group">
			<label for="log-level">
				<span class="label-text">Log Level</span>
				<span class="label-hint">Logging verbosity</span>
			</label>
			<select
				id="log-level"
				bind:value={logLevel}
				on:change={emitApplyDebounced}
			>
				<option value="error">Error</option>
				<option value="warn">Warning</option>
				<option value="info">Info</option>
				<option value="debug">Debug</option>
			</select>
		</div>

		<div class="action-group">
			<button on:click={submitGeo} class="btn-secondary">
				<svg
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z" />
					<circle cx="12" cy="10" r="3" />
				</svg>
				Submit Location
			</button>
		</div>
	</div>
</section>

<style>
.config-panel {
	background: var(--color-surface);
	overflow: hidden;
}

.config-header {
	display: flex;
	align-items: center;
	gap: var(--space-sm);
	padding: var(--space-md) var(--space-lg);
	border-bottom: 1px solid var(--color-border);
	background: linear-gradient(to bottom, var(--color-surface), var(--color-bg));
}

.config-header svg {
	color: var(--color-primary);
	width: 18px;
	height: 18px;
}

.config-header h4 {
	margin: 0;
	font-size: var(--text-base);
	font-weight: var(--font-semibold);
	color: var(--color-text);
}

.config-body {
	padding: var(--space-md) var(--space-lg);
	display: flex;
	flex-direction: column;
	gap: var(--space-md); 
}

.form-group {
	display: flex;
	flex-direction: column;
	gap: var(--space-sm);
}

.form-group label {
	display: flex;
	flex-direction: column;
	gap: 2px;
}

.label-text {
	font-size: var(--text-sm);
	font-weight: var(--font-medium);
	color: var(--color-text);
}

.label-hint {
	font-size: var(--text-xs);
	color: var(--color-text-muted);
}

.toggle-group {
	padding: var(--space-sm) var(--space-md); 
	background: var(--color-bg);
	border-radius: var(--radius-md);
	border: 1px solid var(--color-border);
}

.toggle-label {
	display: flex;
	align-items: center;
	gap: var(--space-sm); 
	cursor: pointer;
	user-select: none;
}

.toggle-input {
	position: absolute;
	opacity: 0;
	width: 0;
	height: 0;
}

.toggle-switch {
	position: relative;
	width: 42px; 
	height: 22px;
	background: var(--color-border);
	border-radius: var(--radius-full);
	transition: background var(--transition-base);
	flex-shrink: 0;
}

.toggle-switch::after {
	content: "";
	position: absolute;
	top: 2px;
	left: 2px;
	width: 18px; 
	height: 18px;
	background: white;
	border-radius: 50%;
	transition: transform var(--transition-base);
	box-shadow: var(--shadow-sm);
}

.toggle-input:checked + .toggle-switch {
	background: var(--color-primary);
}

.toggle-input:checked + .toggle-switch::after {
	transform: translateX(20px); 
}

.toggle-input:focus-visible + .toggle-switch {
	outline: 2px solid var(--color-primary);
	outline-offset: 2px;
}

.toggle-text {
	display: flex;
	flex-direction: column;
	gap: 2px;
}

.toggle-title {
	font-size: var(--text-sm);
	font-weight: var(--font-medium);
	color: var(--color-text);
}

.toggle-desc {
	font-size: var(--text-xs);
	color: var(--color-text-muted);
}


.action-group {
	display: flex;
	flex-direction: column;
	gap: var(--space-sm);
	margin-top: var(--space-sm); 
}

.btn-secondary {
	width: 100%;
	padding: var(--space-sm) var(--space-md); 
	font-weight: var(--font-medium);
	font-size: var(--text-sm); 
	border-radius: var(--radius-md);
	transition: all var(--transition-fast);
	display: flex;
	align-items: center;
	justify-content: center;
	gap: var(--space-sm);
}

.btn-secondary {
	background: var(--color-surface);
	color: var(--color-text);
	border: 1px solid var(--color-border);
}

.btn-secondary:hover {
	background: var(--color-bg);
	border-color: var(--color-border-hover);
}

.btn-secondary:active {
	transform: translateY(0);
}

.config-panel input[type="number"],
.config-panel input[type="url"],
.config-panel select {
	font-size: var(--text-sm);
}

.config-panel select,
.config-panel input[type="url"] {
	background: var(--color-bg);
	color: var(--color-text);
	border: 1px solid var(--color-border);
	-webkit-appearance: none;
	-moz-appearance: none;
	appearance: none;
	padding-right: 2.25rem;
	background-repeat: no-repeat;
	background-position: right 0.6rem center;
	background-size: 1rem;
}

.config-panel input[type="url"] {
	padding: 0.75rem 0.9rem; 
	min-height: 44px;
	line-height: 1.3;
	border-radius: var(--radius-md);
}
</style>
