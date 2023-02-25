/* istanbul ignore file -- @preserve */

export async function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export function expectUnreachable(value: never): never {
	throw new Error(
		`Didn't expect to get here: got a value with case "${value}"`,
	);
}

/**
 * Pads a number on the left with 0's. Accepts numbers that
 * already have more digits than specified in `length`, which
 * just returns the number as a string.
 *
 * @param length minimum number of digits
 * @param num number to pad. Number will be turned positive.
 * @returns padded number. WILL ALWAYS BE POSITIVE.
 */
export function padMin(length: number, num: number) {
	// always positive
	const str = Math.abs(num).toString();

	// already has enough padding
	if (str.length >= length) return str;
	// add padding
	return str.padStart(length, "0");
}

export function reverseMap<K, V>(map: Map<K, V>) {
	return new Map(Array.from(map).reverse());
}

// https://css-tricks.com/restart-css-animation/#aa-update-another-javascript-method-to-restart-a-css-animation
export function resetAnimation(elem: HTMLElement) {
	const prevAnimName = elem.style.animationName;
	elem.style.animationName = "none";
	// trigger a reflow
	void elem.offsetWidth;
	elem.style.animationName = prevAnimName;
}
