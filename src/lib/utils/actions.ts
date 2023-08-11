import type { Action } from "svelte/action";
import { getCSSProp } from "./css";

export const pulse: Action = (
	elem: HTMLElement,
	color = "var(--c-overlay-lightest)",
) => {
	elem.addEventListener("click", handleClick);

	function handleClick() {
		elem.animate(
			[{ backgroundColor: color }, {}],
			getCSSProp("--t-transition", "time") ?? 100,
		);
	}

	return {
		destroy() {
			elem.removeEventListener("click", handleClick);
		},
		update(newColor: string) {
			color = newColor;
		},
	};
};
