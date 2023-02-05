/* eslint-disable @typescript-eslint/ban-ts-comment */

import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { TimerController } from "./timer_controller";

describe("Parses time", () => {
	describe("From milliseconds to h/m/s/ms", () => {
		test.each([
			[100, 0, 0, 0, 0, 100],
			[58_000, 0, 0, 0, 58, 0],
			[60_000, 0, 0, 1, 0, 0],
			[137_000, 0, 0, 2, 17, 0],
			[3_600_000, 0, 1, 0, 0, 0],
			[8_340_000, 0, 2, 19, 0, 0],
			[86_400_000, 1, 0, 0, 0, 0],
			[197_292_332, 2, 6, 48, 12, 332],
		])("Parses %ims to %id, %ih, %im, %is, %ims", (time, d, h, m, s, ms) => {
			expect(TimerController.parseToUnits(time)).toEqual({ d, h, m, s, ms });
		});

		test.each([
			[-100, -0, -0, -0, -0, -100],
			[-58_000, -0, -0, -0, -58, -0],
			[-60_000, -0, -0, -1, -0, -0],
			[-137_000, -0, -0, -2, -17, -0],
			[-3_600_000, -0, -1, -0, -0, -0],
			[-8_340_000, -0, -2, -19, -0, -0],
			[-197_292_332, -2, -6, -48, -12, -332],
		])(
			"Parses a negative time of %ims to %id %ih, %im, %is, %ims",
			(time, d, h, m, s, ms) => {
				expect(TimerController.parseToUnits(time)).toEqual({ d, h, m, s, ms });
			},
		);
	});

	describe("From milliseconds to a clock time", () => {
		describe("Handles trims and padding", () => {
			test("Produces correct time range", () => {
				expect(TimerController.parseToClock(137_020, ["ms", "m"])).toEqual(
					"2:17.020",
				);
			});

			test("Can swap the given time range", () => {
				expect(TimerController.parseToClock(137_020, ["m", "ms"])).toEqual(
					"2:17.020",
				);
			});

			test("Can convert large units into the given range", () => {
				expect(TimerController.parseToClock(5_315_938, ["ms", "m"])).toEqual(
					"88:35.938",
				);
			});

			test("Can allow units to have a longer string when LHS is trimmed", () => {
				expect(TimerController.parseToClock(325_225, ["ms", "s"])).toEqual(
					"325.225",
				);
			});

			test("Adds necessary 0 padding", () => {
				expect(TimerController.parseToClock(123_456, ["ms", "h"])).toEqual(
					"0:02:03.456",
				);
			});

			test("Provides a default time range", () => {
				expect(TimerController.parseToClock(2040)).toEqual("0:00:00:02.040");
			});

			test("Adds 1 to rightmost unit when positive", () => {
				expect(TimerController.parseToClock(137_020, ["s", "m"])).toEqual(
					"2:18",
				);
			});

			test("Does not add 1 when RHS unit is 0", () => {
				expect(TimerController.parseToClock(137_000, ["s", "m"])).toEqual(
					"2:17",
				);
			});

			test("Supports single units, not adding separators", () => {
				expect(TimerController.parseToClock(3_242_521, ["s", "s"])).toEqual(
					"3243",
				);
			});

			test("Can trim down to ms", () => {
				expect(TimerController.parseToClock(100_000_000, ["ms", "ms"])).toEqual(
					"100000000",
				);
			});

			test("Can trim to days only", () => {
				expect(TimerController.parseToClock(200_000_000, ["d", "d"])).toEqual(
					"3",
				);
			});
		});

		describe("Handles negative times", () => {
			test("Adds minus sign when negative", () => {
				expect(TimerController.parseToClock(-19_394)).toEqual(
					"-0:00:00:19.394",
				);
			});

			test("Shifts minus sign when LHS is trimmed", () => {
				expect(TimerController.parseToClock(-38_471, ["ms", "m"])).toEqual(
					"-0:38.471",
				);
			});

			test("When trimmed time is 0, still recognises as negative", () => {
				expect(TimerController.parseToClock(-230, ["s", "m"])).toEqual("-0:00");
			});

			test("Does not add 1 to time when negative", () => {
				expect(TimerController.parseToClock(-23_423, ["s", "m"])).toEqual(
					"-0:23",
				);
			});
		});

		describe("Handles auto mode", () => {
			test.each<[number, string]>([
				[100_000_000, "1:03:46:40.000"],
				[5_324_542, "1:28:44.542"],
				[847_231, "14:07.231"],
				[3742, "3.742"],
			])("Auto-trims each of the units: from %ims to %s", (time, result) => {
				expect(TimerController.parseToClock(time, ["ms", "d"], true)).toEqual(
					result,
				);
			});

			test("Keeps 0 seconds when in the last second of the timer", () => {
				expect(TimerController.parseToClock(123, ["d", "ms"], true)).toEqual(
					"0.123",
				);
			});

			test("Can auto-trim to a restricted set of units", () => {
				expect(
					TimerController.parseToClock(500_000, ["ms", "s"], true),
				).toEqual("500.000");
			});

			test("Can auto-trim from the right side", () => {
				expect(TimerController.parseToClock(653_838, ["s", "d"], true)).toEqual(
					"10:54",
				);
			});
		});
	});
});

describe("Can run", () => {
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

		test("Is started", () => {
			const timer = new TimerController(1000);
			results(timer, "isStarted", [false, true, true, true, true, false]);
		});

		test("Is paused", () => {
			const timer = new TimerController(1000);
			results(timer, "isPaused", [false, false, true, false, false, false]);
		});

		test("Is running", () => {
			const timer = new TimerController(1000);
			results(timer, "isRunning", [false, true, false, true, false, false]);
		});

		test("Is stopped", () => {
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
