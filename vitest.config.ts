import { defineConfig } from "vitest/config";

export default defineConfig({
	define: {
		"import.meta.vitest": "undefined",
	},
	test: {
		coverage: {
			provider: "istanbul",
			reporter: ["text", "json", "html"],
		},
	},
	// test: {
	// 	includeSource: ["src/**/*.{js,ts}"],
	// },
});
