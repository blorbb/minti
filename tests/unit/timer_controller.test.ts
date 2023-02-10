/* eslint-disable @typescript-eslint/ban-ts-comment */

import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { TimerController } from "$lib/utils/timer_controller";

describe("TimerController", () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});
	afterEach(() => {
		vi.useRealTimers();
	});

	describe("Status methods", () => {
		type B = boolean;
		/**
		 *
		 * @param timer
		 * @param funcName name of the status method. starts with `is`
		 * @param values boolean values at each stage of the timer.
		 *
		 * values are:
		 * 1. before start
		 * 2. after start
		 * 3. after pause
		 * 4. after resume
		 * 5. after stop
		 * 6. after reset
		 */
		function results(
			timer: TimerController,
			funcName: keyof TimerController,
			values: [B, B, B, B, B, B],
		) {
			// @ts-ignore
			expect(timer[funcName]()).toEqual(values[0]);
			timer.start();
			// @ts-ignore
			expect(timer[funcName]()).toEqual(values[1]);
			timer.pause();
			// @ts-ignore
			expect(timer[funcName]()).toEqual(values[2]);
			timer.resume();
			// @ts-ignore
			expect(timer[funcName]()).toEqual(values[3]);
			timer.stop();
			// @ts-ignore
			expect(timer[funcName]()).toEqual(values[4]);
			timer.reset();
			// @ts-ignore
			expect(timer[funcName]()).toEqual(values[5]);
		}

		test("Is started: after start, off after reset", () => {
			const timer = new TimerController(1000);
			results(timer, "isStarted", [false, true, true, true, true, false]);
		});

		test("Is paused: after started and paused", () => {
			const timer = new TimerController(1000);
			results(timer, "isPaused", [false, false, true, false, false, false]);
		});

		test("Is running: after started and not paused", () => {
			const timer = new TimerController(1000);
			results(timer, "isRunning", [false, true, false, true, false, false]);
		});

		test("Is stopped: only after calling stop, off after reset", () => {
			const timer = new TimerController(1000);
			results(timer, "isStopped", [false, false, false, false, true, false]);
		});
	});

	describe("Interaction methods", () => {
		test("Returns this", () => {
			const timer = new TimerController(5000);
			expect(timer.start().pause().resume().stop().reset().start()).toBe(timer);
		});

		test("Calling twice does nothing", () => {
			// test that running each method twice still produces the expected results
			function step() {
				vi.advanceTimersByTime(1);
			}

			const timer = new TimerController(10);
			timer.start();
			step(); // step
			timer.start();
			step(); // step
			timer.pause();
			step(); // no
			timer.pause();
			step(); // no
			timer.resume();
			step(); // step
			timer.resume();
			step(); // step
			expect(timer.getTimeElapsed()).toEqual(4);
		});

		test("Pause and resume do not do anything when timer hasn't started", () => {
			const timer = new TimerController(10);
			timer.resume();
			timer.pause();
			timer.resume();
			expect(timer.isStarted()).toEqual(false);
			expect(timer.isPaused()).toEqual(false);
			expect(timer.isRunning()).toEqual(false);
		});

		test("Reset completely resets the timer, with same default duration", () => {
			const timer1 = new TimerController(30);
			timer1.start();
			vi.advanceTimersByTime(40);
			timer1.reset();

			expect(timer1.isStarted()).toEqual(false);
			expect(timer1.isRunning()).toEqual(false);
			expect(timer1.isPaused()).toEqual(false);
			expect(timer1.isStopped()).toEqual(false);
			expect(timer1.getTimeElapsed()).toEqual(0);
			expect(timer1.getTimeRemaining()).toEqual(30);
		});

		test("Does not call onFinish multiple times", () => {
			const mock = vi.fn();
			const timer = new TimerController(1);
			timer.onFinish(() => {
				mock();
			});

			timer.start();
			vi.advanceTimersByTime(10);
			timer.pause().resume().pause().resume();

			expect(mock).toHaveBeenCalledOnce();
		});

		test("Default duration is 0", () => {
			const timer = new TimerController();
			expect(timer.getTimeRemaining()).toEqual(0);
		});
	});

	describe("getTimeElapsed/getTimeRemaining", () => {
		test("Calculates time passed correctly", () => {
			const timer = new TimerController(5000);
			timer.start();
			vi.advanceTimersByTime(2345);
			expect(timer.getTimeElapsed()).toEqual(2345);
			expect(timer.getTimeRemaining()).toEqual(5000 - 2345);
		});

		test("Do not advance when not started", () => {
			const timer = new TimerController(9999);
			vi.advanceTimersByTime(4532);
			expect(timer.getTimeElapsed()).toEqual(0);
			expect(timer.getTimeRemaining()).toEqual(9999);
		});

		test("Is 0 when finished", () => {
			const timer = new TimerController(1000);
			timer.start();
			timer.onFinish(() => {
				expect(timer.getTimeRemaining()).toEqual(0);
			});
			vi.advanceTimersByTime(5000);
		});

		test("Can calculate negative times", () => {
			const timer = new TimerController(10);
			timer.start();
			vi.advanceTimersByTime(50);
			expect(timer.getTimeRemaining()).toEqual(-40);
		});

		test("Does not advance when stopped", () => {
			const timer = new TimerController(10);
			timer.start();
			vi.advanceTimersByTime(4);
			timer.stop();
			vi.advanceTimersByTime(5);
			expect(timer.getTimeElapsed()).toEqual(4);
		});

		describe("Does not count pauses", () => {
			test("Does not include time in pauses", () => {
				const timer = new TimerController(10_000);
				timer.start();
				vi.advanceTimersByTime(2000);
				timer.pause();
				expect(timer.getTimeElapsed()).toEqual(2000);
				vi.advanceTimersByTime(4525);
				expect(timer.getTimeElapsed()).toEqual(2000);
				vi.advanceTimersByTime(138764287);
				expect(timer.getTimeElapsed()).toEqual(2000);
			});

			test("Does not include time in pauses and calculates time elapsed after resume", () => {
				const timer = new TimerController(20_000);
				timer.start();
				vi.advanceTimersByTime(1000);
				timer.pause();
				vi.advanceTimersByTime(1500);
				timer.resume();
				expect(timer.getTimeElapsed()).toEqual(1000);
				vi.advanceTimersByTime(1000);
				expect(timer.getTimeElapsed()).toEqual(2000);
			});

			test("Can handle multiple pauses", () => {
				const timer = new TimerController(100);
				timer.start();
				vi.advanceTimersByTime(10);
				timer.pause();
				vi.advanceTimersByTime(42);
				timer.resume();
				vi.advanceTimersByTime(5);
				timer.pause();
				vi.advanceTimersByTime(2);
				timer.resume();
				vi.advanceTimersByTime(20);
				expect(timer.getTimeElapsed()).toEqual(10 + 5 + 20);
			});
		});
	});
});
