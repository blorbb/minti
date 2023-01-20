export class Timer {
	public static readonly MS_IN_SEC = 1000;
	public static readonly MS_IN_HOUR = 60 * 60 * 1000;
	public static readonly MS_IN_MIN = 60 * 1000;
	public static readonly SECS_IN_MIN = 60;
	public static readonly MINS_IN_HOUR = 60;
	/**
	 * TODO: change to tracking Date() instead of setInterval
	 */
	public static readonly STEP_INTERVAL = 10;

	public static readonly INDEX_TO_UNITS: Record<number, TimeAbbreviations> = {
		0: "ms",
		1: "s",
		2: "m",
		3: "h",
	};
	public static readonly UNITS_TO_INDEX: Record<TimeAbbreviations, number> = {
		ms: 0,
		s: 1,
		m: 2,
		h: 3,
	};

	private interval?: NodeJS.Timer;

	private changeCallback?: (time: number) => void;
	private endCallback?: (time: number) => void;
	/** avoid calling endCallback when timer stays negative */
	private calledEnd = false;

	/**
	 * DO NOT USE outside of set/get `this.time`
	 */
	// eslint-disable-next-line @typescript-eslint/ban-ts-comment
	// @ts-ignore `#time` is set by `time`, which is definitely set in constructor
	#time: number;

	set time(value: number) {
		if (this.changeCallback) this.changeCallback(value);
		if (this.endCallback && value < 0 && !this.calledEnd) {
			this.calledEnd = true;
			this.endCallback(value);
		}
		this.#time = value;
	}

	get time() {
		return this.#time;
	}

	constructor(public readonly initialTime: number) {
		this.time = initialTime;
	}

	public onTimeChange(callback: NonNullable<typeof this.changeCallback>) {
		this.changeCallback = callback;
		this.changeCallback(this.time);
	}

	public onTimerEnd(callback: NonNullable<typeof this.changeCallback>) {
		this.endCallback = callback;
	}

	public start = () => {
		this.time = this.initialTime;
		this.resume();
	};

	public resume = () => {
		if (this.interval) return;
		// setInterval has minimum of 4ms
		this.interval = setInterval(() => {
			this.time -= Timer.STEP_INTERVAL;
		}, Timer.STEP_INTERVAL);
	};

	public pause = () => {
		clearInterval(this.interval);
		this.interval = undefined;
	};

	public reset = () => {
		this.time = this.initialTime;
		this.pause();
	};

	//#region [helper] static methods

	public static parseToUnits(time: number): TimeWithUnits {
		const h = Math.trunc(time / Timer.MS_IN_HOUR);
		const m = Math.trunc(time / Timer.MS_IN_MIN) % Timer.MINS_IN_HOUR;
		const s = Math.trunc(time / Timer.MS_IN_SEC) % Timer.SECS_IN_MIN;
		const ms = time % Timer.MS_IN_SEC;
		return { h, m, s, ms };
	}

	/**
	 *
	 * @param time time in ms
	 * @returns each unit as string, and negative boolean
	 * - hours: no padding
	 * - minutes: 2 digits
	 * - seconds: 2 digits
	 * - milliseconds: 3 digits
	 * - negative: boolean, true if negative
	 *
	 * @deprecated doesn't seem useful, `Timer.parseToClock` is better
	 */
	public static parseToStrings(time: number): TimeWithUnitsAsStrings;
	public static parseToStrings(
		timeWithUnits: TimeWithUnits,
	): TimeWithUnitsAsStrings;
	public static parseToStrings(time: number | TimeWithUnits) {
		let timeWithUnits: TimeWithUnits;
		let negative: boolean;

		if (typeof time === "number") {
			timeWithUnits = Timer.parseToUnits(time);
			negative = time < 0;
		} else {
			timeWithUnits = time;
			negative = Object.values(time).some((value) => value < 0);
		}
		const { h, m, s, ms } = timeWithUnits;

		return {
			h: Math.abs(h).toString(),
			m: Timer.padMin(2, m),
			s: Timer.padMin(2, s),
			ms: Timer.padMin(3, ms),
			negative,
		};
	}

	/**
	 * reorders a UnitRange from smallest to largest unit **in place**
	 * @param range
	 */
	private static standardizeUnitRangeOrder(range: UnitRange) {
		if (Timer.UNITS_TO_INDEX[range[0]] > Timer.UNITS_TO_INDEX[range[1]]) {
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
	public static reduceUnitsToRange(time: number, unitRange: UnitRange) {
		const truncatedTimes = Timer.parseToUnits(time);

		Timer.standardizeUnitRangeOrder(unitRange);
		const smallestUnitIndex = Timer.UNITS_TO_INDEX[unitRange[0]];
		const largestUnitIndex = Timer.UNITS_TO_INDEX[unitRange[1]];

		// reduce large->small
		if (Timer.UNITS_TO_INDEX.h > largestUnitIndex) {
			truncatedTimes.m += truncatedTimes.h * Timer.MINS_IN_HOUR;
			truncatedTimes.h = 0;
		}
		if (Timer.UNITS_TO_INDEX.m > largestUnitIndex) {
			truncatedTimes.s += truncatedTimes.m * Timer.SECS_IN_MIN;
			truncatedTimes.m = 0;
		}
		if (Timer.UNITS_TO_INDEX.s > largestUnitIndex) {
			truncatedTimes.ms += truncatedTimes.s * Timer.MS_IN_SEC;
			truncatedTimes.s = 0;
		}

		const positive = time >= 0;

		// reduce small->large
		// rounds the values UP, so that timer ends as soon as seconds/mins/hrs = 0
		// can't put the truncatedTimes.* === 0 inside the first if statement
		// as it may be -0, we want to set it to 0 for cleanliness
		if (Timer.UNITS_TO_INDEX.ms < smallestUnitIndex) {
			if (truncatedTimes.ms !== 0 && positive) truncatedTimes.s += 1;
			truncatedTimes.ms = 0;
		}
		if (Timer.UNITS_TO_INDEX.s < smallestUnitIndex) {
			if (truncatedTimes.s !== 0 && positive) truncatedTimes.m += 1;
			truncatedTimes.s = 0;
		}
		if (Timer.UNITS_TO_INDEX.m < smallestUnitIndex) {
			if (truncatedTimes.m !== 0 && positive) truncatedTimes.h += 1;
			truncatedTimes.m = 0;
		}

		return truncatedTimes;
	}

	// TODO: add automatic, only shows necessary digits
	public static parseToClock(time: number, unitRange: UnitRange = ["ms", "h"]) {
		const unitTimes = Timer.reduceUnitsToRange(time, unitRange);

		Timer.standardizeUnitRangeOrder(unitRange);

		const largestUnitIndex = Timer.UNITS_TO_INDEX[unitRange[1]];
		const smallestUnitIndex = Timer.UNITS_TO_INDEX[unitRange[0]];

		// const timeStrings = Timer.parseAsStrings(timeUnits);
		// add negative sign if required
		let returnString = time < 0 ? "-" : "";

		// add the necessary padding and separators
		for (let i = largestUnitIndex; i >= smallestUnitIndex; i--) {
			const currentUnit = Timer.INDEX_TO_UNITS[i];

			let separator = "";
			// add separators (:)
			// do not add separators on first iteration
			if (i === largestUnitIndex) separator = "";
			else if (currentUnit !== "ms") separator = ":";
			else separator = ".";

			// add padding
			let padding = 0;
			if (i === largestUnitIndex) padding = 0;
			else if (currentUnit !== "ms") padding = 2;
			else padding = 3;
			const paddedTime = Timer.padMin(padding, unitTimes[currentUnit]);

			returnString += separator + paddedTime;
		}
		return returnString;
	}

	public static padMin(length: number, num: number) {
		// always positive
		const str = Math.abs(num).toString();

		// already has enough padding
		if (str.length >= length) return str;
		// add padding
		return str.padStart(length, "0");
	}

	//#endregion
}

export type TimeAbbreviations = "h" | "m" | "s" | "ms";
export type UnitRange = [TimeAbbreviations, TimeAbbreviations];

export type TimeWithUnits = Record<TimeAbbreviations, number>;
export type TimeWithUnitsAsStrings = Record<TimeAbbreviations, string> & {
	negative: boolean;
};

if (import.meta.vitest) {
	const { it, expect, vi } = import.meta.vitest;
	it.each([
		[100, 0, 0, 0, 100],
		[58000, 0, 0, 58, 0],
		[60000, 0, 1, 0, 0],
		[137000, 0, 2, 17, 0],
		[3600000, 1, 0, 0, 0],
		[8340000, 2, 19, 0, 0],
		[-10, -0, -0, -0, -10],
	])(
		"parses %i ms to %i hour(s), %i minute(s), %i second(s), %i ms",
		(time, h, m, s, ms) => {
			expect(Timer.parseToUnits(time)).toEqual({ h, m, s, ms });
		},
	);

	it.each([
		[99, "0", "00", "00", "099", false],
		[58020, "0", "00", "58", "020", false],
		[60000, "0", "01", "00", "000", false],
		[137000, "0", "02", "17", "000", false],
		[3602000, "1", "00", "02", "000", false],
		[8340000, "2", "19", "00", "000", false],
		[-10, "0", "00", "00", "010", true],
	])(
		"parses %i ms to a string of %s:%s:%s.%s",
		(time, h, m, s, ms, negative) => {
			expect(Timer.parseToStrings(time)).toEqual({ h, m, s, ms, negative });
		},
	);

	it.each([
		[{ h: 1, m: 2, s: 54, ms: 3 }, "1", "02", "54", "003", false],
		[{ h: 0, m: -2, s: -1, ms: 0 }, "0", "02", "01", "000", true],
	])(
		"converts timeWithUnits to strings correctly",
		(timeWithUnits, h, m, s, ms, negative) => {
			expect(Timer.parseToStrings(timeWithUnits)).toEqual({
				h,
				m,
				s,
				ms,
				negative,
			});
		},
	);

	it("counts down time correctly", () => {
		const timer = new Timer(2000);
		vi.useFakeTimers();
		timer.start();
		vi.advanceTimersByTime(2000);
		expect(timer.time).toEqual(0);
		vi.useRealTimers();
	});

	it.each<[number, TimeAbbreviations, TimeAbbreviations, string]>([
		[137020, "ms", "m", "2:17.020"],
		[137020, "m", "s", "2:18"],
		[-1000, "s", "s", "-1"],
		[-800, "m", "s", "-0:00"],
		[-137020, "s", "m", "-2:17"],
	])(
		"converts time to a clock time",
		(time, upperRange, lowerRange, clockString) => {
			expect(Timer.parseToClock(time, [upperRange, lowerRange])).toEqual(
				clockString,
			);
		},
	);

	it.each([
		[2, 1, "01"],
		[2, 32, "32"],
		[3, 1234, "1234"],
	])("pads to %i places: %i to %s", (length, num, str) => {
		expect(Timer.padMin(length, num)).toEqual(str);
	});
}
