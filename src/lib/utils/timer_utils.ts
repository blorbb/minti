/**
 * Utility functions to be used by the other timer functions.
 *
 * Implicitly tested through the other functions' tests.
 */

import { expectUnreachable } from "./misc";

export type TimeAbbreviations = "d" | "h" | "m" | "s" | "ms";
export type UnitRange = [TimeAbbreviations, TimeAbbreviations];
export type TimeWithUnits = Record<TimeAbbreviations, number>;

export const constants = {
	MS_IN_SEC: 1000,
	SECS_IN_MIN: 60,
	MINS_IN_HOUR: 60,
	HOURS_IN_DAY: 24,
	get MS_IN_MIN() {
		return this.SECS_IN_MIN * this.MS_IN_SEC;
	},
	get MS_IN_HOUR() {
		return this.MINS_IN_HOUR * this.MS_IN_MIN;
	},
	get MS_IN_DAY() {
		return this.HOURS_IN_DAY * this.MS_IN_HOUR;
	},
} as const;

export const unitStrings = {
	DAYS: ["d", "day", "days"],
	HOURS: ["h", "hr", "hrs", "hour", "hours"],
	MINS: ["m", "min", "mins", "minute", "minutes"],
	SECS: ["s", "sec", "secs", "second", "seconds"],
	MS: [
		"ms",
		"milli",
		"millis",
		"millisec",
		"millisecs",
		"millisecond",
		"milliseconds",
	],
	get VALID_STRINGS() {
		return [
			...this.DAYS,
			...this.HOURS,
			...this.MINS,
			...this.SECS,
			...this.MS,
		];
	},
	stringToUnit(token: string): TimeAbbreviations {
		// no clue why istanbul thinks that the SECS branch is not
		// being covered. only this is comment working to ignore it
		/* istanbul ignore else -- @preserve */
		if (this.DAYS.includes(token)) return "d";
		else if (this.HOURS.includes(token)) return "h";
		else if (this.MINS.includes(token)) return "m";
		else if (this.SECS.includes(token)) return "s";
		else if (this.MS.includes(token)) return "ms";
		else {
			/* istanbul ignore next -- @preserve */
			throw new Error("Invalid unit");
		}
	},
	// TODO maybe add configurable decimal separator
	UNIT_SEPARATOR: ":",
	// DO NOT set this `as const`
	// for some reason it makes Array.prototype.includes only
	// allow strings that are already in the array
};

export const convert = {
	daysToMs: (days: number) => days * constants.MS_IN_DAY,
	hoursToMs: (hours: number) => hours * constants.MS_IN_HOUR,
	minsToMs: (minutes: number) => minutes * constants.MS_IN_MIN,
	secsToMs: (seconds: number) => seconds * constants.MS_IN_SEC,
	msToDays: (ms: number) => ms / constants.MS_IN_DAY,
	msToHours: (ms: number) => ms / constants.MS_IN_HOUR,
	msToMins: (ms: number) => ms / constants.MS_IN_MIN,
	msToSecs: (ms: number) => ms / constants.MS_IN_SEC,
	timeUnitToMs(num: number, fromUnit: TimeAbbreviations) {
		switch (fromUnit) {
			case "d":
				return this.daysToMs(num);
			case "h":
				return this.hoursToMs(num);
			case "m":
				return this.minsToMs(num);
			case "s":
				return this.secsToMs(num);
			case "ms":
				return num;
		}
		/* istanbul ignore next -- @preserve */
		expectUnreachable(fromUnit);
	},
	msToTimeUnit(ms: number, toUnit: TimeAbbreviations) {
		// case "d" is not used by any other functions
		// don't know why ignoring that case isn't working with
		// the coverage tests
		/* istanbul ignore next -- @preserve */
		switch (toUnit) {
			case "d": {
				return this.msToDays(ms);
			}
			case "h":
				return this.msToHours(ms);
			case "m":
				return this.msToMins(ms);
			case "s":
				return this.msToSecs(ms);
			case "ms":
				return ms;
		}
		/* istanbul ignore next -- @preserve */
		expectUnreachable(toUnit);
	},
	convert(num: number, fromUnit: TimeAbbreviations, toUnit: TimeAbbreviations) {
		const ms = this.timeUnitToMs(num, fromUnit);
		return this.msToTimeUnit(ms, toUnit);
	},
} as const;

/**
 * `ms = 0`, `d = 4`
 */
export const order = {
	// not `as const` as TS doesn't know that
	// the strings are TimeAbbreviations
	INDEX_TO_UNITS: {
		0: "ms",
		1: "s",
		2: "m",
		3: "h",
		4: "d",
	} as Record<number, TimeAbbreviations>,
	UNITS_TO_INDEX: {
		ms: 0,
		s: 1,
		m: 2,
		h: 3,
		d: 4,
	} as Record<TimeAbbreviations, number>,
};
