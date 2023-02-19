module.exports = {
	root: true,
	parser: "@typescript-eslint/parser",
	extends: [
		"eslint:recommended",
		"plugin:@typescript-eslint/recommended",
		"prettier",
	],
	plugins: ["svelte3", "@typescript-eslint"],
	ignorePatterns: ["*.cjs"],
	overrides: [
		{ files: ["*.svelte"], processor: "svelte3/svelte3" },
		// https://github.com/Chatie/eslint-config/issues/45#issuecomment-1003990077
		{ files: ["*.svelte", "*.ts"], rules: { "no-undef": "off" } },
	],
	settings: {
		"svelte3/typescript": () => require("typescript"),
	},
	parserOptions: {
		sourceType: "module",
		ecmaVersion: 2020,
	},
	env: {
		browser: true,
		es2017: true,
		node: true,
	},
};
