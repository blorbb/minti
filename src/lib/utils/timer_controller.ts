export class TimerController {
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
	public start(startTime = Date.now()) {
		if (this.isStarted()) {
			return this;
		}
		this.clear();
		this.#startTimestamp = startTime;
		this.resume(startTime);
		return this;
	}

	public resume(resumeTime = Date.now()) {
		if (!this.isPaused() || this.isStopped()) {
			return this;
		}
		this.#lastResumeTimestamp = resumeTime;
		this.startFinishTimer();
		return this;
	}

	public pause() {
		if (this.#lastResumeTimestamp === undefined || !this.isStarted()) {
			return this;
		}
		this.#accumulatedTimeElapsed += Date.now() - this.#lastResumeTimestamp;
		this.#lastResumeTimestamp = undefined;
		this.stopFinishTimer();
		return this;
	}

	public reset(duration = this.#duration) {
		this.clear();
		this.#duration = duration;
		return this;
	}

	public stop() {
		this.pause();
		this.#endTimestamp = Date.now();
		this.stopFinishTimer();
		return this;
	}

	private clear() {
		this.#startTimestamp = undefined;
		this.#endTimestamp = undefined;
		this.#lastResumeTimestamp = undefined;
		this.#accumulatedTimeElapsed = 0;
		this.#finished = false;
		this.stopFinishTimer();
		return this;
	}

	/**
	 * Increase or decrease duration of the timer.
	 *
	 * @param ms Milliseconds to increase the duration by.
	 * Use a negative number to decrease duration.
	 */
	public addDuration(ms: number) {
		if (this.isStopped()) {
			return this;
		}

		// require always non negative
		this.#duration = Math.max(0, this.#duration + ms);

		// reset finish timer
		this.stopFinishTimer();
		this.startFinishTimer();

		return this;
	}
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

	public isFinished() {
		return this.getTimeRemaining() <= 0;
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

	/**
	 * @returns Duration of the timer in ms
	 */
	public getTimerDuration() {
		return this.#duration;
	}
	//#endregion

	//#region finish
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
		// check if already finished
		// do not call onFinishCallback again
		if (this.#finished) return;
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
	//#endregion
}
