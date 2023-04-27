import { sveltekit } from "@sveltejs/kit/vite";
import type { UserConfig } from "vite";
import { internalIpV4 } from "internal-ip";

const mobile =
	process.env.TAURI_PLATFORM === "android" ||
	process.env.TAURI_PLATFORM === "ios";

const config: UserConfig = {
	plugins: [sveltekit()],
	css: {
		preprocessorOptions: {
			scss: {
				additionalData: '@use "src/variables.scss" as *;',
			},
		},
	},
	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	// prevent vite from obscuring rust errors
	clearScreen: false,
	// tauri expects a fixed port, fail if that port is not available
	server: {
		host: mobile ? "0.0.0.0" : false,
		port: 5173,
		hmr: mobile
			? {
					protocol: "ws",
					host: await internalIpV4(),
					port: 5183,
			  }
			: undefined,
		strictPort: true,
	},
	// to make use of `TAURI_DEBUG` and other env variables
	// https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
	envPrefix: ["VITE_", "TAURI_"],
	build: {
		// Tauri supports es2021
		target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
		// don't minify for debug builds
		minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
		// produce sourcemaps for debug builds
		sourcemap: !!process.env.TAURI_DEBUG,
	},
};

export default config;
