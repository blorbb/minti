import tippy from "tippy.js";
import type { Props as TippyProps, DefaultProps } from "tippy.js";

/**
 * Svelte action for tooltips.
 *
 * @param options
 * Options for the tooltip.
 *
 * - `content`: The text shown in the tooltip. Can either be a string
 *   or an array of strings, which will be separated by new lines.
 * - `theme`: Name of the theme to be used. This is applied to the
 *   Tippy.js property (to be used by CSS) and also automatically adds
 *   some other properties, as defined by `THEME_PROPS`. Default theme
 *   is `"dark"`.
 * - `enabled`: Whether the tooltip will show on hover.
 * - `tippy`: All other tippy props.
 */
export function tooltip(target: HTMLElement, options: TooltipOptions) {
	function getTippyProps(options: TooltipOptions): Partial<TippyProps> {
		// set content prop
		const content = Array.isArray(options.text)
			? options.text.join("\n")
			: options.text;

		const theme = options.theme ?? (tippy.defaultProps.theme as TooltipThemes);
		// set extra props depending on theme
		const themeProps: Partial<TippyProps> = THEME_PROPS[theme];

		return {
			...themeProps,
			...options.tippy,
			theme,
			content,
		};
	}

	// create tippy instance
	const instance = tippy(target, {
		...getTippyProps(options),
		// to inspect with dev tools
		// trigger: "click",
	});

	const enabled = options.enabled ?? DEFAULT_OPTIONS.enabled;
	if (!enabled) {
		instance.hide();
		instance.disable();
	}

	return {
		destroy() {
			instance.destroy();
		},
		update(options: TooltipOptions) {
			const props = getTippyProps(options);
			instance.setProps(props);

			// edit visibility
			const enabled = options.enabled ?? DEFAULT_OPTIONS.enabled;
			if (enabled) {
				instance.enable();
			} else {
				instance.hide();
				instance.disable();
			}
		},
	};
}

export const THEME_PROPS: Record<TooltipThemes, Partial<TippyProps>> = {
	dark: {
		animation: "scale",
		inertia: true,
	},
	error: {
		animation: "scale",
		inertia: true,
	},
	translucent: {
		animation: "fade",
		arrow: false,
	},
};

export type TooltipThemes = "dark" | "error" | "translucent";

export type TooltipOptions = {
	text: string | string[];
	theme?: TooltipThemes;
	enabled?: boolean;
	tippy?: Omit<Partial<TippyProps>, "content" | "theme">;
};

const DEFAULT_OPTIONS: Omit<TooltipOptions, keyof DefaultProps | "text"> = {
	enabled: true,
};

tippy.setDefaultProps({
	theme: "dark",
	placement: "top",
	offset: [0, 4],
	delay: 100,
	appendTo: () => document.fullscreenElement ?? document.body,
});
