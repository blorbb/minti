import { expectUnreachable } from "./misc";

/**
 * first capturing group = number
 *
 * second capturing group = unit
 */
const CSS_UNIT_REGEX = /(?<value>-?[0-9.]+)(?<unit>[a-zA-Z]*)/;

export type CSSTypes = "string" | "time" | "length";

/**
 * Retrieves a CSS custom property set on the `:root` element.
 *
 * @param prop Name of the CSS property. Starts with `--`.
 * @param type
 * The type of the value to get. Automatically parses + converts units.
 *
 * Defaults to `"string"`, which just gives the value of the property,
 * trimmed and lowercase. If the value is empty, `""` will be returned.
 * Use `getProp(...) || "default"` to set a default.
 *
 * If a `type` other than `"string"` is given, the function will return
 * a number. If unable to parse the value, `null` is returned. Use
 * `getProp(...)  ?? default` to set a default.
 * - `type` = `"time"`, returns a number in `ms`
 * - `type` = `"length"`, returns a number in `px` (NOT IMPLEMENTED YET)
 */
export function getCSSProp(prop: string, type?: "string"): string;
export function getCSSProp(
	prop: string,
	type: Exclude<CSSTypes, "string">,
): number | null;
export function getCSSProp(
	prop: string,
	type: CSSTypes = "string",
): string | number | null {
	const value = getComputedStyle(document.documentElement)
		.getPropertyValue(prop)
		.trim()
		.toLocaleLowerCase();

	if (type === "string") return value;

	// get number and unit of value
	const match = value.match(CSS_UNIT_REGEX);
	if (match === null) return null;

	const numString = match.groups?.value;
	const unit = match.groups?.unit;
	if (numString === undefined || unit === undefined) return null;
	const num = +numString;
	if (isNaN(num)) return null;

	switch (type) {
		case "time": {
			if (unit === "s") {
				return num * 1000;
			} else if (unit === "ms" || unit === "") {
				// no unit is technically not a valid CSS time
				// but just assuming it's ms in this case
				return num;
			}
			return null;
		}
		case "length": {
			// TODO
			throw new Error("TODO: LENGTH CASE");
		}
		default:
			expectUnreachable(type);
	}
}
