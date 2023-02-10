/**
 * formats a time (number in ms) to
 * - clock time (string with format d:hh:mm:ss.msmsms)
 * - units: converts time to days + hours + minutes + seconds + milliseconds
 */

import { padMin } from "./misc";
import type { TimeAbbreviations } from "./timer_controller";
import {
	constants,
	convert,
	order,
	unitStrings,
	type TimeWithUnits,
	type UnitRange,
} from "./timer_utils";

/**
 * Class with two public methods `toUnits` and `toClock`.
 * The rest are helper methods for these two functions.
 *
 * Documentation for the public methods are at the bottom with the exports.
 */
class FormatTime {
	public static toUnits(time: number): TimeWithUnits {
		const d = Math.trunc(convert.msToDays(time));
		const h = Math.trunc(convert.msToHours(time)) % constants.HOURS_IN_DAY;
		const m = Math.trunc(convert.msToMins(time)) % constants.MINS_IN_HOUR;
		const s = Math.trunc(convert.msToSecs(time)) % constants.SECS_IN_MIN;
		const ms = time % constants.MS_IN_SEC;
		return { d, h, m, s, ms };
	}

	public static toClock(
		time: number,
		unitRange: UnitRange = ["ms", "d"],
		auto = false,
	) {
		const unitTimes = FormatTime.reduceUnitsToRange(time, unitRange);

		FormatTime.reorderUnitRange(unitRange);

		const largestUnit = unitRange[1];
		const largestUnitIndex = order.UNITS_TO_INDEX[largestUnit];
		const smallestUnit = unitRange[0];
		const smallestUnitIndex = order.UNITS_TO_INDEX[smallestUnit];

		// add negative sign if required
		let returnString = time < 0 ? "-" : "";

		// add the necessary padding and separators
		/** Whether the current unit is the first to be shown */
		let isFirstUnit = true;

		for (
			let unitIndex = largestUnitIndex;
			unitIndex >= smallestUnitIndex;
			unitIndex--
		) {
			const currentUnit = order.INDEX_TO_UNITS[unitIndex];
			if (
				auto &&
				unitTimes[currentUnit] === 0 &&
				isFirstUnit &&
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
			// do not add separators before the first number
			if (isFirstUnit) separator = "";
			else if (currentUnit !== "ms") separator = unitStrings.UNIT_SEPARATOR;
			else separator = ".";

			// add padding
			let padding = 0;
			if (isFirstUnit) padding = 0;
			else if (currentUnit !== "ms") padding = 2;
			else padding = 3;
			const paddedTime = padMin(padding, unitTimes[currentUnit]);

			returnString += separator + paddedTime;
			isFirstUnit = false;
		}
		return returnString;
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
	 * converts a time to the specified range of units available
	 *
	 * @param time time in ms
	 * @param unitRange range of units to use
	 * @returns object of times converted into units
	 */
	private static reduceUnitsToRange(time: number, unitRange: UnitRange) {
		const truncatedTimes = FormatTime.toUnits(time);

		FormatTime.reorderUnitRange(unitRange);
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

		// check that none of the units have overflown
		// e.g. for a time of 59sec 999ms and a range excluding ms
		// seconds would be 60
		// need to push this to the larger unit as 1 minute 0 seconds
		// need to check:
		// the unit value >= its overflow, and the larger unit
		// is still within the unitRange
		const orderedTimes = FormatTime.toOrderedMap(truncatedTimes);

		for (const [unit, value] of orderedTimes) {
			const nextUnit = this.nextLargerUnit(unit);

			// check that it is within the unitRange
			if (
				nextUnit === null ||
				order.UNITS_TO_INDEX[nextUnit] > largestUnitIndex
			) {
				break;
			}

			// get the number required for overflow
			// e.g. current unit = hours
			// convert 1 day to hours = 24 hours
			const unitLimit = convert.convert(1, nextUnit, unit);
			if (value >= unitLimit) {
				// round this value
				// in case it somehow is more than 2x greater
				// than the limit, e.g. 49 hours => 2x
				const limitExceedingMultiplier = Math.floor(value / unitLimit);
				// convert this unit down to within its limit
				orderedTimes.set(unit, value % unitLimit);

				// increment next value
				// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
				const nextValue = orderedTimes.get(nextUnit)!;
				orderedTimes.set(nextUnit, nextValue + limitExceedingMultiplier);
			}
		}

		return Object.fromEntries(orderedTimes);
	}

	/**
	 * Gets the next larger unit from the one given. Returns `null` if
	 * the given unit is already the largest one there is ("d")
	 *
	 * @param unit any time abbreviation EXCLUDING days ("d")
	 */
	private static nextLargerUnit(unit: TimeAbbreviations) {
		const nextUnitIndex = order.UNITS_TO_INDEX[unit] + 1;

		if (!(nextUnitIndex in order.INDEX_TO_UNITS)) return null;
		return order.INDEX_TO_UNITS[nextUnitIndex];
	}

	/**
	 * Converts a TimeWithUnits object to a Map, which can be iterated upon.
	 * Goes from milliseconds to days.
	 *
	 * @param timeWithUnits
	 */
	private static toOrderedMap(timeWithUnits: TimeWithUnits) {
		const map = new Map<TimeAbbreviations, number>();
		map.set("ms", timeWithUnits.ms);
		map.set("s", timeWithUnits.s);
		map.set("m", timeWithUnits.m);
		map.set("h", timeWithUnits.h);
		map.set("d", timeWithUnits.d);

		return map;
	}
}

/**
 * Converts a time in ms to hours, minutes, seconds and milliseconds.
 *
 * @param time time in ms
 * @returns time converted to days, hours, minutes, seconds and milliseconds.
 * Access the times easily with
 * ```ts
 * { d, h, m, s, ms } = formatTimeToUnits(time)
 * ```
 */
export const formatTimeToUnits = FormatTime.toUnits;

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
export const formatTimeToClock = FormatTime.toClock;
