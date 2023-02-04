import { defineConfig } from "vitest/config";

export default defineConfig({
	define: {
		"import.meta.vitest": "undefined",
	},
	test: {
		coverage: {
			provider: "c8",
			reporter: ["text", "json", "html"],
		},
	},
	// test: {
	// 	includeSource: ["src/**/*.{js,ts}"],
	// },
});
