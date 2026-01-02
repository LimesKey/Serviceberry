<script lang="ts">
import type { LogItem } from "../lib/types";
export let items: LogItem[] = [];

let container: HTMLUListElement | null = null;
$: if (container) container.scrollTop = 0;

const levelConfig = {
	debug: { color: "#64748b", label: "DEBUG" },
	info: { color: "#3b82f6", label: "INFO" },
	warn: { color: "#f59e0b", label: "WARN" },
	error: { color: "#ef4444", label: "ERROR" },
};
</script>

<section class="logs-panel">
	<div class="logs-header">
		<svg
			width="20"
			height="20"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
		>
			<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
			<polyline points="14 2 14 8 20 8" />
			<line x1="16" y1="13" x2="8" y2="13" />
			<line x1="16" y1="17" x2="8" y2="17" />
			<polyline points="10 9 9 9 8 9" />
		</svg>
		<h4>Activity Log</h4>
		<span class="log-count">{items.length}</span>
	</div>

	<ul bind:this={container} class="logs-list">
		{#if items.length === 0}
			<li class="empty-state">
				<svg
					width="48"
					height="48"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path
						d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
					/>
					<polyline points="14 2 14 8 20 8" />
				</svg>
				<p>No activity yet</p>
				<span>Logs will appear here</span>
			</li>
		{:else}
			{#each items as log (log.id)}
				<li class="log-item" data-level={log.level}>
					<span class="log-icon" aria-hidden="true"
						>{levelConfig[log.level]}</span
					>
					<div class="log-content">
						<div class="log-header-row">
							<span class="log-time">{log.ts}</span>
							<span
								class="log-badge"
								style="background-color: {levelConfig[log.level].color}"
							>
								{levelConfig[log.level].label}
							</span>
						</div>
						<p class="log-message">{log.msg}</p>
					</div>
				</li>
			{/each}
		{/if}
	</ul>
</section>

<style>
.logs-panel {
	background: var(--color-surface);
	display: flex;
	flex-direction: column;
	overflow: hidden;
	flex: 1;
}

.logs-header {
	display: flex;
	align-items: center;
	gap: var(--space-sm);
	padding: var(--space-md) var(--space-lg);
	border-bottom: 1px solid var(--color-border);
	background: linear-gradient(to bottom, var(--color-surface), var(--color-bg));
}

.logs-header svg {
	color: var(--color-primary);
	width: 18px;
	height: 18px;
}

.logs-header h4 {
	margin: 0;
	font-size: var(--text-base);
	font-weight: var(--font-semibold);
	color: var(--color-text);
	flex: 1;
}

.log-count {
	font-size: var(--text-xs);
	font-weight: var(--font-medium);
	padding: 2px 8px;
	background: var(--color-primary-light);
	color: var(--color-primary);
	border-radius: var(--radius-full);
}

.logs-list {
	list-style: none;
	margin: 0;
	padding: 0;
	overflow-y: auto;
	flex: 1;
}

.empty-state {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	padding: var(--space-2xl);
	text-align: center;
	color: var(--color-text-muted);
	min-height: 200px;
}

.empty-state svg {
	opacity: 0.3;
	margin-bottom: var(--space-md);
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

.log-item {
	display: flex;
	gap: var(--space-sm);
	padding: var(--space-sm) var(--space-md);
	border-bottom: 1px solid var(--color-border);
	transition: background var(--transition-fast);
	animation: slideIn 0.2s ease;
}

@keyframes slideIn {
	from {
		opacity: 0;
		transform: translateY(-4px);
	}
	to {
		opacity: 1;
		transform: translateY(0);
	}
}

.log-item:hover {
	background: var(--color-bg);
}

.log-item[data-level="error"] {
	background: rgba(239, 68, 68, 0.05);
	border-left: 3px solid #ef4444;
}

.log-item[data-level="warn"] {
	background: rgba(245, 158, 11, 0.05);
	border-left: 3px solid #f59e0b;
}

.log-icon {
	font-size: var(--text-base);
	flex-shrink: 0;
	width: 20px;
	text-align: center;
}

.log-content {
	flex: 1;
	min-width: 0;
}

.log-header-row {
	display: flex;
	align-items: center;
	gap: var(--space-sm);
	margin-bottom: 4px;
}

.log-time {
	font-family: var(--font-mono);
	font-size: var(--text-xs);
	color: var(--color-text-muted);
}

.log-badge {
	font-size: 10px;
	font-weight: var(--font-bold);
	color: white;
	padding: 2px 6px;
	border-radius: var(--radius-sm);
	text-transform: uppercase;
	letter-spacing: 0.5px;
}

.log-message {
	font-size: var(--text-sm);
	color: var(--color-text);
	margin: 0;
	word-wrap: break-word;
	line-height: 1.5;
}

/* Scrollbar for logs */
.logs-list::-webkit-scrollbar {
	width: 6px;
}

.logs-list::-webkit-scrollbar-track {
	background: var(--color-surface);
}

.logs-list::-webkit-scrollbar-thumb {
	background: var(--color-border);
	border-radius: var(--radius-full);
}

.logs-list::-webkit-scrollbar-thumb:hover {
	background: var(--color-border-hover);
}
</style>
