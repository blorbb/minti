import adapter from "@sveltejs/adapter-static";
import preprocess from "svelte-preprocess";
import { vitePreprocess } from "@sveltejs/kit/vite";

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://kit.svelte.dev/docs/integrations#preprocessors
	// for more information about preprocessors
	preprocess: [
		preprocess({
			scss: {
				prependData: '@use "src/variables.scss" as *;',
			},
		}),
		// needed to use classes in svelte files ??? might be able to remove
		vitePreprocess(),
	],

	kit: {
		adapter: adapter(),
	},
};

export default config;
