import { get } from "svelte/store";
import { padMin } from "./misc";
import { settings } from "./stores";
import { constants } from "./timer_utils";

/**
 * Formats the time to an end time.
 * @param msAway Number of milliseconds ahead
 * @returns Relative time.
 * - If it is on the same day, displays the time (e.g. `12:45 pm`).
 * - If it is on the next day, displays "tmr." and the time (e.g. `tmr. 12:45 pm`).
 * - If it is within the next 7 days, displays the day and time (e.g. `Sat. 12:45 pm`).
 * - If it is more than 7 days away, displays the date and time (e.g. `2023-03-04 12:45pm`).
 */
export function formatRelativeTime(msAway: number) {
	const currentDate = new Date();
	const endDate = new Date(currentDate.getTime() + msAway);

	const endTime = toTime(endDate, get(settings).timeFormat);
	const dayDifference = differenceInDays(currentDate, endDate);

	// same day: only show the end time
	if (dayDifference === 0) {
		return endTime;
	} else if (dayDifference < 7) {
		// say "tmr" or the day of the week
		let relativeDay: string;
		if (dayDifference === 1) relativeDay = "tmr";
		else relativeDay = DAY_STRINGS.SHORT[endDate.getDay()];

		return `${relativeDay}. ${endTime}`;
	}
	return `${toLocalISODate(endDate)} ${endTime}`;
}

/**
 * List of strings for each day of the week. Mostly for use with `date.getDay()`.
 *
 * Example:
 * ```ts
 * const date = new Date(Date.UTC(2023, 3, 5))
 * DAY_STRINGS.SHORT[date.getDay()] === "Sun" // true
 * ```
 */
export const DAY_STRINGS = {
	SHORT: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
	LONG: [
		"Sunday",
		"Monday",
		"Tuesday",
		"Wednesday",
		"Thursday",
		"Friday",
		"Saturday",
	],
} as const;

/**
 * Gets the local date as string in the format YYYY-MM-DD.
 * Does not include the time.
 *
 * @param date
 */
export function toLocalISODate(date: Date) {
	return `${date.getFullYear()}-${padMin(2, date.getMonth() + 1)}-${padMin(
		2,
		date.getDate(),
	)}`;
}

export function toTime(date: Date, type: "12h" | "24h") {
	return (
		date
			.toLocaleTimeString("en-US", {
				hour12: type === "12h",
				hour: "numeric",
				minute: "numeric",
			})
			.toLocaleLowerCase()
			// format uses narrow no-break space - replace with a regular space
			.replace(/\s/g, " ")
			// midnight is 24:00
			.replace(/^24:/, "00:")
	);
}

export function isSameDay(date1: Date, date2: Date) {
	return toLocalISODate(date1) === toLocalISODate(date2);
}

export function startOfDay(date: Date) {
	const newDate = new Date(date);
	newDate.setHours(0, 0, 0, 0);
	return newDate;
}

export function differenceInDays(date1: Date, date2: Date) {
	let earlierDate = startOfDay(date1);
	let laterDate = startOfDay(date2);
	// sort so earlierDate < laterDate
	[earlierDate, laterDate] = [earlierDate, laterDate].sort(
		(a, b) => a.getTime() - b.getTime(),
	);

	return Math.round(
		(laterDate.getTime() - earlierDate.getTime()) / constants.MS_IN_DAY,
	);
}
