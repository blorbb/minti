export class Timer {
	//#region [definitions] constants
	public static readonly MS_IN_SEC = 1000;
	public static readonly MS_IN_HOUR = 60 * 60 * 1000;
	public static readonly MS_IN_MIN = 60 * 1000;
	public static readonly SECS_IN_MIN = 60;
	public static readonly MINS_IN_HOUR = 60;

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
	//#endregion

	#startTimestamp?: number;
	#endTimestamp?: number;
	/**
	 * number of ms that have passed in the timer, accounting
	 * for pauses. Only updated upon pause, use `getTimeElapsed()`
	 * to get the current time elapsed.
	 */
	#accumulatedTimeElapsed = 0;
	/**
	 * time since the timer started or was unpaused
	 */
	#lastResumeTimestamp?: number;
	#duration: number;

	constructor(duration: number) {
		this.#duration = duration;
	}

	// TODO next
	private endCallback?: (time: number) => void;
	public onTimerEnd(callback: NonNullable<typeof this.endCallback>) {
		this.endCallback = callback;
	}

	//#region interaction methods
	public start = () => {
		if (this.isStarted()) {
			return this;
		}
		this.clear();
		this.#startTimestamp = Date.now();
		this.resume();
		return this;
	};

	public resume = () => {
		if (!this.isPaused() || this.isStopped()) {
			return this;
		}
		this.#lastResumeTimestamp = Date.now();
		return this;
	};

	public pause = () => {
		if (this.#lastResumeTimestamp === undefined || !this.isStarted()) {
			return this;
		}
		this.#accumulatedTimeElapsed += Date.now() - this.#lastResumeTimestamp;
		this.#lastResumeTimestamp = undefined;
		return this;
	};

	public reset = (duration = this.#duration) => {
		this.clear();
		this.#duration = duration;
		return this;
	};

	public stop = () => {
		this.pause();
		this.#endTimestamp = Date.now();
	};

	private clear = () => {
		this.#startTimestamp = undefined;
		this.#endTimestamp = undefined;
		this.#lastResumeTimestamp = undefined;
		this.#accumulatedTimeElapsed = 0;
		return this;
	};
	//#endregion

	//#region status methods
	/**
	 * Timer started, including being paused or having ended.
	 * Use `isRunning()` to check whether the timer is still running.
	 * @returns whether the timer is started
	 */
	public isStarted() {
		return this.#startTimestamp !== undefined;
	}

	/**
	 * Time has started and is paused, not including being stopped.
	 * Use `isStopped()` to check whether the timer has stopped.
	 * @returns whether the timer is paused
	 */
	public isPaused() {
		return (
			this.isStarted() &&
			this.#lastResumeTimestamp === undefined &&
			!this.isStopped()
		);
	}

	public isRunning() {
		return this.isStarted() && !this.isStopped() && !this.isPaused();
	}

	/**
	 * Timer has stopped/ended, not including pauses.
	 * Use `isPaused()` to check whether the timer has paused.
	 * @returns whether the timer is stopped
	 */
	public isStopped() {
		return this.#endTimestamp !== undefined;
	}

	public getTimeElapsed() {
		if (!this.isStarted()) {
			return 0;
		} else if (this.isPaused()) {
			return this.#accumulatedTimeElapsed;
		} else if (this.isStopped()) {
			// might change
			return this.#accumulatedTimeElapsed;
		}
		// currently running
		// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
		const timeSinceResume = Date.now() - this.#lastResumeTimestamp!;
		return this.#accumulatedTimeElapsed + timeSinceResume;
	}

	public getTimeRemaining() {
		return this.#duration - this.getTimeElapsed();
	}
	//#endregion

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
