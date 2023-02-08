import { defineConfig } from "vitest/config";
import { resolve } from "path";

export default defineConfig({
	define: {
		"import.meta.vitest": "undefined",
	},
	test: {
		coverage: {
			provider: "istanbul",
			reporter: ["text", "json", "html"],
			reportsDirectory: "./tests/coverage",
		},
		include: ["tests/unit/**/*.{test,spec}.{js,ts}"],
	},
	// https://stackoverflow.com/a/74374936
	resolve: {
		alias: {
			$lib: resolve(__dirname, "./src/lib"),
		},
	},
});
