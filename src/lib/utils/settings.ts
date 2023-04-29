export type CSSSetting = {
	title: string;
	description?: string;
	variableName: string;
	defaultValue: string;
};

export const variableSettings: CSSSetting[] = [
	{
		title: "Background colour",
		variableName: "--background",
		defaultValue: "#111",
	},
	{
		title: "Scrollbar width",
		variableName: "--l-scrollbar__width",
		defaultValue: "0px",
	},
	{
		title: "Navbar width",
		variableName: "--l-navbar__width",
		defaultValue: "3rem",
	},
	{
		title: "Navbar background",
		variableName: "--navbar__background",
		defaultValue: "rgba(0 0 0 / 0.719)",
	},
	{
		title: "Progress bar border width",
		description: "Only applies when the background progress bar mode is used.",
		variableName: "--l-progress-bar--bg__border-width",
		defaultValue: "0px",
	},
	{
		title: "Timer padding",
		description: "How far apart different timers are.",
		variableName: "--l-timer-list__padding",
		defaultValue: "0.5rem",
	},
	{
		title: "Timer roundness",
		description: "Border radius of each timer.",
		variableName: "--l-timer-box__border-radius",
		defaultValue: "0.5rem",
	},
];

export function createStyleSheet() {
	const styleTag = document.createElement("style");
	styleTag.id = "configurable-style";
	document.getElementsByTagName("head")[0].appendChild(styleTag);
	updateStyleSheet();
}

/**
 * Requires that `createStyleSheet` has previously been called.
 */
export function updateStyleSheet() {
	const styleTag = document.getElementById(
		"configurable-style",
	) as HTMLStyleElement;

	const styles: Record<string, string> = {};
	for (const setting of variableSettings) {
		styles[setting.variableName] =
			localStorage.getItem(`setting.style.${setting.variableName}`) ??
			setting.defaultValue;
	}

	styleTag.innerText = `:root{${Object.keys(styles).reduce(
		(accum, key) => accum + `${key}:${styles[key]};`,
		"",
	)}}`;
}
