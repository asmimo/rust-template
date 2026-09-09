import type { Alpine } from "alpinejs";

declare global {
	var Alpine: Alpine;

	interface Console {
		inspect: (...data: unknown[]) => void;
	}
}
