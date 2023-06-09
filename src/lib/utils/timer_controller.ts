import { get, writable, type Writable } from "svelte/store";
import { order, type TimeAbbreviations, type UnitRange } from "./timer_utils";
import { formatTimeToStrings } from "./time_formatter";
import { formatRelativeTime } from "./date";
import { settings } from "./stores";
type TimeDisplay = [TimeAbbreviations, string][];

type TimerStatus = {
	/** Timer has started, including being paused or having ended. */
	started: boolean;
	/** Timer has started and is paused. Does not include being stopped. */
	paused: boolean;
	/** Timer has stopped, not including pauses. */
	stopped: boolean;
	/** Timer has finished (time remaining <= 0). */
	finished: boolean;
	/** Timer is started and not paused or ended. */
	running: boolean;
};

export class TimerController {
	startTimestamp?: number;
	endTimestamp?: number;
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
	duration: Writable<number>;
	status: Writable<TimerStatus>;
	display: TimerDisplay;

	constructor(duration = 0) {
		this.duration = writable(duration);

		this.status = writable({
			started: false,
			paused: false,
			stopped: false,
			finished: false,
			running: false,
		});

		this.display = new TimerDisplay(this);
	}

	//#region interaction methods
	public start() {
		if (get(this.status).started) {
			return this;
		}
		this.clear();
		this.startTimestamp = Date.now();
		this.status.update((s) => ({ ...s, started: true }));
		this.resume();

		this.display.updateEndTime();
		this.display.updateArray();
		this.display.startInterval();
		return this;
	}

	public resume() {
		if (
			!get(this.status).started &&
			(!get(this.status).paused || get(this.status).stopped)
		) {
			return this;
		}

		this.#lastResumeTimestamp = Date.now();
		this.startFinishTimer();

		this.status.update((s) => ({ ...s, paused: false, running: true }));
		this.display.startInterval();
		return this;
	}

	public pause() {
		if (
			this.#lastResumeTimestamp === undefined ||
			!get(this.status).started ||
			get(this.status).stopped
		) {
			return this;
		}
		this.#accumulatedTimeElapsed += Date.now() - this.#lastResumeTimestamp;
		this.#lastResumeTimestamp = undefined;
		this.stopFinishTimer();

		this.status.update((s) => ({ ...s, paused: true, running: false }));
		this.display.stopInterval();
		return this;
	}

	public reset(duration = get(this.duration)) {
		this.clear();
		this.duration.set(duration);

		this.status.set({
			started: false,
			paused: false,
			running: false,
			finished: false,
			stopped: false,
		});
		this.display.stopInterval();
		return this;
	}

	public stop() {
		this.pause();
		this.endTimestamp = Date.now();
		this.stopFinishTimer();

		this.status.update((s) => ({
			...s,
			stopped: true,
			running: true,
			paused: false,
		}));
		return this;
	}

	private clear() {
		this.startTimestamp = undefined;
		this.endTimestamp = undefined;
		this.#lastResumeTimestamp = undefined;
		this.#accumulatedTimeElapsed = 0;
		this.stopFinishTimer();

		this.status.set({
			started: false,
			paused: false,
			running: false,
			finished: false,
			stopped: false,
		});
		return this;
	}

	/**
	 * Increase or decrease duration of the timer.
	 *
	 * @param ms Milliseconds to increase the duration by.
	 * Use a negative number to decrease duration.
	 */
	public addDuration(ms: number) {
		if (get(this.status).stopped) {
			return this;
		}

		// require always non negative
		this.duration.update((d) => Math.max(0, d + ms));
		this.display.updateEndTime();
		this.display.updateArray();

		// reset finish timer
		this.stopFinishTimer();
		this.startFinishTimer();
		return this;
	}
	//#endregion

	//#region statuses
	/**
	 * Amount of time that has elapsed while running.
	 * Does not include pauses.
	 * @returns total elapsed time of the timer in ms
	 */
	public getTimeElapsed() {
		if (!get(this.status).started) {
			return 0;
		} else if (get(this.status).paused || get(this.status).stopped) {
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
		return get(this.duration) - this.getTimeElapsed();
	}
	//#endregion

	//#region finish
	/**
	 * Timeout used to wait until the timer finishes to send
	 * the `onFinish` event. Only use through `startFinishTimer()`
	 * and `endFinishTimer()`.
	 */
	private completionTimeout?: NodeJS.Timeout;

	/**
	 * Starts the `completionTimeout` timer waiting for the timer to
	 * reach 0. Activates the `onFinish` callback function. The timeout
	 * needs to be stopped if the timer is paused, using
	 * `stopFinishTimer()`.
	 */
	private startFinishTimer() {
		const timeRemaining = this.getTimeRemaining();
		if (timeRemaining <= 0) this.setFinished();
		else {
			// start timer to check again
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
		if (!this.completionTimeout || get(this.status).finished) return;
		clearTimeout(this.completionTimeout);
		this.completionTimeout = undefined;
	}

	/**
	 * Sets `#finished` to `true` and activates the callback in
	 * `onFinish(callback)`.
	 */
	private setFinished() {
		// check if already finished
		// do not call onFinishCallback again
		if (get(this.status).finished) return;

		this.status.update((s) => ({ ...s, finished: true }));

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
	//#endregion
}

class TimerDisplay {
	public timeArray: Writable<[TimeAbbreviations, string][]>;
	private updateTimeInterval: number;
	private _updateInterval?: NodeJS.Timer;

	private unitRange: UnitRange;
	private autoTrim: boolean;

	public endTime: Writable<string>;
	private _endTimeUpdateInterval?: NodeJS.Timer;

	private controller: TimerController;

	constructor(controller: TimerController) {
		this.controller = controller;
		this.unitRange = get(settings).timerUnitRange;
		this.autoTrim = get(settings).autoTrimTimerDisplay;
		this.updateTimeInterval = get(settings).timerUpdateInterval;

		this.endTime = writable("");
		this.timeArray = writable([]);
	}

	public updateArray() {
		const times = formatTimeToStrings(
			this.controller.getTimeRemaining(),
			this.unitRange,
			this.autoTrim,
		);

		// don't format this as a string as there are different
		// classes for the different parts of the time
		const timeArray: TimeDisplay = Array.from(
			order.recordToMap(times),
		).reverse();

		// check that all digits are 0
		// if so, remove the negative 0
		if (timeArray.every(([, timeStr]) => +timeStr == 0)) {
			// omit the negative 0
			let firstTimeStr = timeArray[0][1];
			if (firstTimeStr[0] === "-") firstTimeStr = firstTimeStr.slice(1);

			timeArray[0][1] = firstTimeStr;
		}

		this.timeArray.set(timeArray);
	}

	public startInterval() {
		if (this._updateInterval) this.stopInterval();

		// status should be updated which already calls an update
		// shouldn't need, but uncomment if needed
		// timerDisplay.update();
		this._updateInterval = setInterval(
			() => this.updateArray(),
			this.updateTimeInterval,
		);

		this.stopEndTimeInterval();
	}

	stopInterval() {
		clearInterval(this._updateInterval);
		this._updateInterval = undefined;

		this._startEndTimeInterval();
	}

	updateEndTime() {
		this.endTime.set(formatRelativeTime(this.controller.getTimeRemaining()));
	}
	private _startEndTimeInterval() {
		if (this._endTimeUpdateInterval) this.stopEndTimeInterval();
		// status should be updated which already calls an update
		// shouldn't need, but uncomment if needed
		// this.updateEndTime();
		this._endTimeUpdateInterval = setInterval(() => this.updateEndTime(), 2000);
	}
	stopEndTimeInterval() {
		clearInterval(this._endTimeUpdateInterval);
		this._endTimeUpdateInterval = undefined;
	}
}
