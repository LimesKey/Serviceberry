export function formatTimeWithoutSeconds(date: Date): string {
	return date.toLocaleTimeString("en-US", {
		hour: "numeric",
		minute: "2-digit",
		hour12: true,
	});
}

export function getRelativeTime(timestamp: number): string {
	const now = Date.now();
	const diff = now - timestamp;
	const seconds = Math.floor(diff / 1000);
	const minutes = Math.floor(seconds / 60);
	const hours = Math.floor(minutes / 60);
	const days = Math.floor(hours / 24);

	if (seconds < 10) return "just now";
	if (seconds < 60) return `${seconds}s ago`;
	if (minutes < 60) return `${minutes}m ago`;
	if (hours < 24) return `${hours}h ago`;
	return `${days}d ago`;
}

// Determine if a channel is 2.4GHz or 5GHz

export function getChannelBand(channel: number): "2.4GHz" | "5GHz" | "unknown" {
	if (channel >= 1 && channel <= 14) return "2.4GHz";
	if (channel >= 32 && channel <= 177) return "5GHz";
	return "unknown";
}

// Get the frequency in MHz for a given channel

export function getChannelFrequency(channel: number): number {
	// 2.4 GHz channels
	if (channel >= 1 && channel <= 13) {
		return 2412 + (channel - 1) * 5;
	}
	if (channel === 14) {
		return 2484;
	}

	// 5 GHz channels
	if (channel >= 32 && channel <= 177) {
		return 5000 + channel * 5;
	}

	return 0;
}
