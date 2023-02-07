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
		if (this.DAYS.includes(token)) return "d";
		else if (this.HOURS.includes(token)) return "h";
		else if (this.MINS.includes(token)) return "m";
		else if (this.SECS.includes(token)) return "s";
		else if (this.MS.includes(token)) return "ms";
		else throw new Error("Invalid unit");
	},
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
			default:
				expectUnreachable(fromUnit);
		}
	},
	msToTimeUnit(ms: number, toUnit: TimeAbbreviations) {
		switch (toUnit) {
			case "d":
				return this.msToDays(ms);
			case "h":
				return this.msToHours(ms);
			case "m":
				return this.msToMins(ms);
			case "s":
				return this.msToSecs(ms);
			case "ms":
				return ms;
			default:
				expectUnreachable(toUnit);
		}
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

class ParseToString {
	/**
	 * Converts a time in ms to hours, minutes, seconds and milliseconds.
	 *
	 * @param time time in ms
	 * @returns time converted to days, hours, minutes, seconds and milliseconds.
	 * Access the times easily with
	 * ```ts
	 * { d, h, m, s, ms } = Timer.parseToUnits(time)
	 * ```
	 */
	public static parseToUnits(time: number): TimeWithUnits {
		const d = Math.trunc(convert.msToDays(time));
		const h = Math.trunc(convert.msToHours(time)) % constants.HOURS_IN_DAY;
		const m = Math.trunc(convert.msToMins(time)) % constants.MINS_IN_HOUR;
		const s = Math.trunc(convert.msToSecs(time)) % constants.SECS_IN_MIN;
		const ms = time % constants.MS_IN_SEC;
		return { d, h, m, s, ms };
	}

	/**
	 * reorders a UnitRange from smallest to largest unit **in place**
	 * @param range
	 */
	private static reorderUnitRange(range: UnitRange) {
		if (order.UNITS_TO_INDEX[range[0]] > order.UNITS_TO_INDEX[range[1]]) {
			range.reverse();
		}
	}

	/**
	 * Converts time in ms to a string with a specified range of units
	 * to convert to.
	 *
	 * @param time time in ms
	 * @param unitRange range of units to use. E.g. to convert to units from
	 * minutes to milliseconds, use `["ms", "m"]`.
	 * @param auto whether to automatically remove 0's. `unitRange` becomes
	 * the maximum allowed range of units.
	 * @returns time as a string, converted to the units specified, separated
	 * by `:` (or `.` for milliseconds).
	 */
	public static parseToClock(
		time: number,
		unitRange: UnitRange = ["ms", "d"],
		auto = false,
	) {
		const unitTimes = ParseToString.reduceUnitsToRange(time, unitRange);

		ParseToString.reorderUnitRange(unitRange);

		const largestUnitIndex = order.UNITS_TO_INDEX[unitRange[1]];
		const smallestUnitIndex = order.UNITS_TO_INDEX[unitRange[0]];

		// const timeStrings = Timer.parseAsStrings(timeUnits);
		// add negative sign if required
		let returnString = time < 0 ? "-" : "";

		// add the necessary padding and separators
		/** Whether the current unit is the first to be shown */
		let firstIteration = true;
		for (let i = largestUnitIndex; i >= smallestUnitIndex; i--) {
			const currentUnit = order.INDEX_TO_UNITS[i];
			if (
				auto &&
				unitTimes[currentUnit] === 0 &&
				firstIteration &&
				currentUnit !== "s"
			) {
				// remove all 0's from the left side
				// still keep 0's if they are between units
				// (from the firstIteration check)
				// always keep seconds, never just have ms alone
				// so that the last second will be `0.123`
				continue;
			}

			let separator = "";
			// add separators (:)
			// do not add separators on first iteration
			if (firstIteration) separator = "";
			else if (currentUnit !== "ms") separator = ":";
			else separator = ".";

			// add padding
			let padding = 0;
			if (firstIteration) padding = 0;
			else if (currentUnit !== "ms") padding = 2;
			else padding = 3;
			const paddedTime = ParseToString.padMin(padding, unitTimes[currentUnit]);

			returnString += separator + paddedTime;
			firstIteration = false;
		}
		return returnString;
	}

	/**
	 * converts a time to the specified range of units available
	 *
	 * @param time time in ms
	 * @param unitRange range of units to use
	 * @returns object of times converted into units
	 */
	private static reduceUnitsToRange(time: number, unitRange: UnitRange) {
		const truncatedTimes = ParseToString.parseToUnits(time);

		ParseToString.reorderUnitRange(unitRange);
		const smallestUnitIndex = order.UNITS_TO_INDEX[unitRange[0]];
		const largestUnitIndex = order.UNITS_TO_INDEX[unitRange[1]];

		// reduce large->small
		if (order.UNITS_TO_INDEX["d"] > largestUnitIndex) {
			truncatedTimes.h += convert.convert(truncatedTimes.d, "d", "h");
			truncatedTimes.d = 0;
		}
		if (order.UNITS_TO_INDEX["h"] > largestUnitIndex) {
			truncatedTimes.m += convert.convert(truncatedTimes.h, "h", "m");
			truncatedTimes.h = 0;
		}
		if (order.UNITS_TO_INDEX["m"] > largestUnitIndex) {
			truncatedTimes.s += convert.convert(truncatedTimes.m, "m", "s");
			truncatedTimes.m = 0;
		}
		if (order.UNITS_TO_INDEX["s"] > largestUnitIndex) {
			truncatedTimes.ms += convert.convert(truncatedTimes.s, "s", "ms");
			truncatedTimes.s = 0;
		}

		const notNegative = time >= 0;

		// reduce small->large
		// rounds the values UP, so that timer ends as soon as seconds/mins/hrs/days = 0
		// can't put the truncatedTimes.* === 0 inside the first if statement
		// as it may be -0, we want to set it to 0 for cleanliness
		if (order.UNITS_TO_INDEX["ms"] < smallestUnitIndex) {
			if (truncatedTimes.ms !== 0 && notNegative) truncatedTimes.s += 1;
			truncatedTimes.ms = 0;
		}
		if (order.UNITS_TO_INDEX["s"] < smallestUnitIndex) {
			if (truncatedTimes.s !== 0 && notNegative) truncatedTimes.m += 1;
			truncatedTimes.s = 0;
		}
		if (order.UNITS_TO_INDEX["m"] < smallestUnitIndex) {
			if (truncatedTimes.m !== 0 && notNegative) truncatedTimes.h += 1;
			truncatedTimes.m = 0;
		}
		if (order.UNITS_TO_INDEX["h"] < smallestUnitIndex) {
			if (truncatedTimes.h !== 0 && notNegative) truncatedTimes.d += 1;
			truncatedTimes.h = 0;
		}

		return truncatedTimes;
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
	private static padMin(length: number, num: number) {
		// always positive
		const str = Math.abs(num).toString();

		// already has enough padding
		if (str.length >= length) return str;
		// add padding
		return str.padStart(length, "0");
	}
}

export const parseFromTime = {
	toUnits: ParseToString.parseToUnits,
	toClock: ParseToString.parseToClock,
};

/**
 * TODO
 * export const parse = {
 *     fromTimeToUnits: ...,
 *     fromClockToUnits: ...,
 *     fromStringToTIme: ...,
 * }
 * change time_parser.ts to hold all of these
 * or maybe
 * export const parseFrom = {
 *     timeToUnits: ...,
 *     ...
 * }
 */
