/* eslint-disable @typescript-eslint/ban-ts-comment */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TimerController } from "./timer_controller";

describe("Parses time", () => {
	describe("From milliseconds to h/m/s/ms", () => {
		it.each([
			[100, 0, 0, 0, 100],
			[58000, 0, 0, 58, 0],
			[60000, 0, 1, 0, 0],
			[137000, 0, 2, 17, 0],
			[3600000, 1, 0, 0, 0],
			[8340000, 2, 19, 0, 0],
		])("parses %ims to %ih, %im, %is, %ims", (time, h, m, s, ms) => {
			expect(TimerController.parseToUnits(time)).toEqual({ h, m, s, ms });
		});

		it.each([
			[-100, -0, -0, -0, -100],
			[-58000, -0, -0, -58, -0],
			[-60000, -0, -1, -0, -0],
			[-137000, -0, -2, -17, -0],
			[-3600000, -1, -0, -0, -0],
			[-8340000, -2, -19, -0, -0],
		])(
			"parses a negative time of %ims to %ih, %im, %is, %ims",
			(time, h, m, s, ms) => {
				expect(TimerController.parseToUnits(time)).toEqual({ h, m, s, ms });
			},
		);
	});

	describe("From milliseconds to a clock time", () => {
		describe("Handles trims and padding", () => {
			it("Produces correct time range", () => {
				expect(TimerController.parseToClock(137020, ["ms", "m"])).toEqual(
					"2:17.020",
				);
			});

			it("Can swap the given time range", () => {
				expect(TimerController.parseToClock(137020, ["m", "ms"])).toEqual(
					"2:17.020",
				);
			});

			it("Can convert large units into the given range", () => {
				expect(TimerController.parseToClock(5315938, ["ms", "m"])).toEqual(
					"88:35.938",
				);
			});

			it("Can allow units to have a longer string when LHS is trimmed", () => {
				expect(TimerController.parseToClock(325225, ["ms", "s"])).toEqual(
					"325.225",
				);
			});

			it("Adds necessary 0 padding", () => {
				expect(TimerController.parseToClock(123456, ["ms", "h"])).toEqual(
					"0:02:03.456",
				);
			});

			it("Provides a default time range", () => {
				expect(TimerController.parseToClock(2040)).toEqual("0:00:02.040");
			});

			it("Adds 1 to rightmost unit when positive", () => {
				expect(TimerController.parseToClock(137020, ["s", "m"])).toEqual(
					"2:18",
				);
			});

			it("Does not add 1 when RHS unit is 0", () => {
				expect(TimerController.parseToClock(137000, ["s", "m"])).toEqual(
					"2:17",
				);
			});

			it("Supports single units, not adding separators", () => {
				expect(TimerController.parseToClock(3242521, ["s", "s"])).toEqual(
					"3243",
				);
			});
		});

		describe("Handles negative times", () => {
			it("Adds minus sign when negative", () => {
				expect(TimerController.parseToClock(-19394)).toEqual("-0:00:19.394");
			});

			it("Shifts minus sign when LHS is trimmed", () => {
				expect(TimerController.parseToClock(-38471, ["ms", "m"])).toEqual(
					"-0:38.471",
				);
			});

			it("When trimmed time is 0, still recognises as negative", () => {
				expect(TimerController.parseToClock(-230, ["s", "m"])).toEqual("-0:00");
			});

			it("Does not add 1 to time when negative", () => {
				expect(TimerController.parseToClock(-23423, ["s", "m"])).toEqual(
					"-0:23",
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

		it("is started", () => {
			const timer = new TimerController(1000);
			results(timer, "isStarted", [false, true, true, true, true, false]);
		});

		it("is paused", () => {
			const timer = new TimerController(1000);
			results(timer, "isPaused", [false, false, true, false, false, false]);
		});

		it("is running", () => {
			const timer = new TimerController(1000);
			results(timer, "isRunning", [false, true, false, true, false, false]);
		});

		it("is stopped", () => {
			const timer = new TimerController(1000);
			results(timer, "isStopped", [false, false, false, false, true, false]);
		});
	});

	// TODO add more tests for the timer counting down correctly
});
