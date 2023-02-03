export class TimerController {
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

	constructor(duration = 0) {
		this.#duration = duration;
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
		this.startFinishTimer();
		return this;
	};

	public pause = () => {
		if (this.#lastResumeTimestamp === undefined || !this.isStarted()) {
			return this;
		}
		this.#accumulatedTimeElapsed += Date.now() - this.#lastResumeTimestamp;
		this.#lastResumeTimestamp = undefined;
		this.stopFinishTimer();
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
		this.stopFinishTimer();
	};

	private clear = () => {
		this.#startTimestamp = undefined;
		this.#endTimestamp = undefined;
		this.#lastResumeTimestamp = undefined;
		this.#accumulatedTimeElapsed = 0;
		this.#finished = false;
		this.stopFinishTimer();
		return this;
	};
	//#endregion

	//#region status methods
	/**
	 * Timer started, including being paused or having ended.
	 * Use `isRunning()` to check whether the timer is still running.
	 *
	 * @returns whether the timer is started
	 */
	public isStarted() {
		return this.#startTimestamp !== undefined;
	}

	/**
	 * Time has started and is paused, not including being stopped.
	 * Use `isStopped()` to check whether the timer has stopped.
	 *
	 * @returns whether the timer is paused
	 */
	public isPaused() {
		return (
			this.isStarted() &&
			this.#lastResumeTimestamp === undefined &&
			!this.isStopped()
		);
	}

	/**
	 * Timer is started and not paused or ended
	 *
	 * @returns whether the timer is ticking
	 */
	public isRunning() {
		return this.isStarted() && !this.isStopped() && !this.isPaused();
	}

	/**
	 * Timer has stopped/ended, not including pauses.
	 * Use `isPaused()` to check whether the timer has paused.
	 *
	 * @returns whether the timer is stopped
	 */
	public isStopped() {
		return this.#endTimestamp !== undefined;
	}

	/**
	 * Amount of time that has elapsed while running.
	 * Does not include pauses.
	 * @returns total elapsed time of the timer in ms
	 */
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

	/**
	 * @returns time in ms remaining for the timer to reach 0ms
	 */
	public getTimeRemaining() {
		return this.#duration - this.getTimeElapsed();
	}
	//#endregion

	/**
	 * Timeout used to wait until the timer finishes to send
	 * the `onFinish` event. Only use through `startFinishTimer()`
	 * and `endFinishTimer()`.
	 */
	private completionTimeout?: NodeJS.Timeout;
	/** Whether the timer has passed 0. Set `true` using `setFinished()` */
	#finished = false;

	/**
	 * Starts the `completionTimeout` timer waiting for the timer to
	 * reach 0. Activates the `onFinish` callback function. The timeout
	 * needs to be stopped if the timer is paused, using
	 * `stopFinishTimer()`.
	 */
	private startFinishTimer() {
		if (this.#finished) return;
		if (this.completionTimeout) this.stopFinishTimer();
		// check if already finished
		const timeRemaining = this.getTimeRemaining();
		if (timeRemaining <= 0) this.setFinished();
		// start timer to check again
		else {
			this.completionTimeout = setTimeout(() => {
				this.startFinishTimer();
			}, timeRemaining);
		}
	}

	/**
	 * Stops the `completionTimeout` timer, so that it can restart
	 * once the timer resumes again.
	 */
	private stopFinishTimer() {
		if (!this.completionTimeout || this.#finished) return;
		clearTimeout(this.completionTimeout);
		this.completionTimeout = undefined;
	}

	/**
	 * Sets `#finished` to `true` and activates the callback in
	 * `onFinish(callback)`.
	 */
	private setFinished() {
		this.#finished = true;
		if (this.onFinishCallback) this.onFinishCallback();
	}

	/** Callback defined by `onFinish(callback) */
	private onFinishCallback?: () => void;
	/**
	 * Only called when the timer reaches 0. Not called when the timer
	 * is manually stopped with `timer.stop()`.
	 */
	public onFinish(callback: () => void) {
		this.onFinishCallback = callback;
	}

	//#region [helper] static methods

	/**
	 * Converts a time in ms to hours, minutes, seconds and milliseconds.
	 *
	 * @param time time in ms
	 * @returns time converted to hours, minutes, seconds and milliseconds.
	 * Access the times easily with
	 * ```ts
	 * { h, m, s, ms } = Timer.parseToUnits(time)
	 * ```
	 */
	public static parseToUnits(time: number): TimeWithUnits {
		const h = Math.trunc(time / TimerController.MS_IN_HOUR);
		const m =
			Math.trunc(time / TimerController.MS_IN_MIN) %
			TimerController.MINS_IN_HOUR;
		const s =
			Math.trunc(time / TimerController.MS_IN_SEC) %
			TimerController.SECS_IN_MIN;
		const ms = time % TimerController.MS_IN_SEC;
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
			timeWithUnits = TimerController.parseToUnits(time);
			negative = time < 0;
		} else {
			timeWithUnits = time;
			negative = Object.values(time).some((value) => value < 0);
		}
		const { h, m, s, ms } = timeWithUnits;

		return {
			h: Math.abs(h).toString(),
			m: TimerController.padMin(2, m),
			s: TimerController.padMin(2, s),
			ms: TimerController.padMin(3, ms),
			negative,
		};
	}

	/**
	 * reorders a UnitRange from smallest to largest unit **in place**
	 * @param range
	 */
	private static standardizeUnitRangeOrder(range: UnitRange) {
		if (
			TimerController.UNITS_TO_INDEX[range[0]] >
			TimerController.UNITS_TO_INDEX[range[1]]
		) {
			range.reverse();
		}
	}

	// TODO: add automatic, only shows necessary digits
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
		unitRange: UnitRange = ["ms", "h"],
		auto = false,
	) {
		const unitTimes = TimerController.reduceUnitsToRange(time, unitRange);

		TimerController.standardizeUnitRangeOrder(unitRange);

		const largestUnitIndex = TimerController.UNITS_TO_INDEX[unitRange[1]];
		const smallestUnitIndex = TimerController.UNITS_TO_INDEX[unitRange[0]];

		// const timeStrings = Timer.parseAsStrings(timeUnits);
		// add negative sign if required
		let returnString = time < 0 ? "-" : "";

		// add the necessary padding and separators
		/** Whether the current unit is the first to be shown */
		let firstIteration = true;
		for (let i = largestUnitIndex; i >= smallestUnitIndex; i--) {
			const currentUnit = TimerController.INDEX_TO_UNITS[i];
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
			const paddedTime = TimerController.padMin(
				padding,
				unitTimes[currentUnit],
			);

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
		const truncatedTimes = TimerController.parseToUnits(time);

		TimerController.standardizeUnitRangeOrder(unitRange);
		const smallestUnitIndex = TimerController.UNITS_TO_INDEX[unitRange[0]];
		const largestUnitIndex = TimerController.UNITS_TO_INDEX[unitRange[1]];

		// reduce large->small
		if (TimerController.UNITS_TO_INDEX.h > largestUnitIndex) {
			truncatedTimes.m += truncatedTimes.h * TimerController.MINS_IN_HOUR;
			truncatedTimes.h = 0;
		}
		if (TimerController.UNITS_TO_INDEX.m > largestUnitIndex) {
			truncatedTimes.s += truncatedTimes.m * TimerController.SECS_IN_MIN;
			truncatedTimes.m = 0;
		}
		if (TimerController.UNITS_TO_INDEX.s > largestUnitIndex) {
			truncatedTimes.ms += truncatedTimes.s * TimerController.MS_IN_SEC;
			truncatedTimes.s = 0;
		}

		const positive = time >= 0;

		// reduce small->large
		// rounds the values UP, so that timer ends as soon as seconds/mins/hrs = 0
		// can't put the truncatedTimes.* === 0 inside the first if statement
		// as it may be -0, we want to set it to 0 for cleanliness
		if (TimerController.UNITS_TO_INDEX.ms < smallestUnitIndex) {
			if (truncatedTimes.ms !== 0 && positive) truncatedTimes.s += 1;
			truncatedTimes.ms = 0;
		}
		if (TimerController.UNITS_TO_INDEX.s < smallestUnitIndex) {
			if (truncatedTimes.s !== 0 && positive) truncatedTimes.m += 1;
			truncatedTimes.s = 0;
		}
		if (TimerController.UNITS_TO_INDEX.m < smallestUnitIndex) {
			if (truncatedTimes.m !== 0 && positive) truncatedTimes.h += 1;
			truncatedTimes.m = 0;
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

	//#endregion
}

export type TimeAbbreviations = "h" | "m" | "s" | "ms";
export type UnitRange = [TimeAbbreviations, TimeAbbreviations];

export type TimeWithUnits = Record<TimeAbbreviations, number>;
export type TimeWithUnitsAsStrings = Record<TimeAbbreviations, string> & {
	negative: boolean;
};
