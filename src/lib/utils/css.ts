/**
 * first capturing group = number
 *
 * second capturing group = unit
 */
const CSS_UNIT_REGEX = /(?<value>-?[0-9.]+)(?<unit>[a-zA-Z]*)/;

type CSSTypes = "string" | "time" | "length";

function getProp(prop: string, type?: "string"): string | null;
function getProp(prop: string, type: "time" | "length"): number | null;
function getProp(
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
		default: {
			const check: never = type;
			throw new Error("Unhandled case for type " + check);
		}
	}
}

export const CSSProps = {
	get: getProp,
	// might add more later, e.g. refresh
};
