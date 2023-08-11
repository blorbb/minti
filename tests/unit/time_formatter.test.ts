import { describe, test, expect } from "vitest";
import {
	formatTimeToUnits,
	formatTimeToClock,
} from "$lib/utils/time_formatter";

describe("Formats time", () => {
	describe("From milliseconds to d/h/m/s/ms", () => {
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
			expect(formatTimeToUnits(time)).toEqual({ d, h, m, s, ms });
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
				expect(formatTimeToUnits(time)).toEqual({ d, h, m, s, ms });
			},
		);
	});

	describe("From milliseconds to a clock time", () => {
		describe("Handles trims and padding", () => {
			test("Produces correct time range", () => {
				expect(formatTimeToClock(137_020, ["ms", "m"])).toEqual("2:17.020");
			});

			test("Can swap the given time range", () => {
				expect(formatTimeToClock(137_020, ["m", "ms"])).toEqual("2:17.020");
			});

			test("Can convert large units into the given range", () => {
				expect(formatTimeToClock(5_315_938, ["ms", "m"])).toEqual("88:35.938");
			});

			test("Can allow units to have a longer string when LHS is trimmed", () => {
				expect(formatTimeToClock(325_225, ["ms", "s"])).toEqual("325.225");
			});

			test("Adds necessary 0 padding", () => {
				expect(formatTimeToClock(123_456, ["ms", "h"])).toEqual("0:02:03.456");
			});

			test("Provides a default time range", () => {
				expect(formatTimeToClock(2040)).toEqual("0:00:00:02.040");
			});

			test("Adds 1 to rightmost unit when positive", () => {
				expect(formatTimeToClock(137_020, ["s", "m"])).toEqual("2:18");
			});

			test("Does not add 1 when RHS unit is 0", () => {
				expect(formatTimeToClock(137_000, ["s", "m"])).toEqual("2:17");
			});

			test("Supports single units, not adding separators", () => {
				expect(formatTimeToClock(3_242_521, ["s", "s"])).toEqual("3243");
			});

			test("Can trim down to ms", () => {
				expect(formatTimeToClock(100_000_000, ["ms", "ms"])).toEqual(
					"100000000",
				);
			});

			test("Can trim to days only", () => {
				expect(formatTimeToClock(200_000_000, ["d", "d"])).toEqual("3");
			});

			test("Number of days is 2 (not 3) when time is exactly 2 days", () => {
				expect(formatTimeToClock(172_800_000, ["d", "d"])).toEqual("2");
			});
		});

		describe("Handles negative times", () => {
			test("Adds minus sign when negative", () => {
				expect(formatTimeToClock(-19_394)).toEqual("-0:00:00:19.394");
			});

			test("Shifts minus sign when LHS is trimmed", () => {
				expect(formatTimeToClock(-38_471, ["ms", "m"])).toEqual("-0:38.471");
			});

			test("When trimmed time is 0, still recognises as negative", () => {
				expect(formatTimeToClock(-230, ["s", "m"])).toEqual("-0:00");
			});

			test("Does not add 1 to time when negative", () => {
				expect(formatTimeToClock(-23_423, ["s", "m"])).toEqual("-0:23");
			});
		});

		describe("Handles auto mode", () => {
			test.each<[number, string]>([
				[100_000_000, "1:03:46:40.000"],
				[5_324_542, "1:28:44.542"],
				[847_231, "14:07.231"],
				[3742, "3.742"],
			])("Auto-trims each of the units: from %ims to %s", (time, result) => {
				expect(formatTimeToClock(time, ["ms", "d"], true)).toEqual(result);
			});

			test("Keeps 0 seconds when in the last second of the timer", () => {
				expect(formatTimeToClock(123, ["d", "ms"], true)).toEqual("0.123");
			});

			test("Can auto-trim to a restricted set of units", () => {
				expect(formatTimeToClock(500_000, ["ms", "s"], true)).toEqual(
					"500.000",
				);
			});

			test("Can auto-trim from the right side", () => {
				expect(formatTimeToClock(653_838, ["s", "d"], true)).toEqual("10:54");
			});

			describe("Edge cases when a larger unit ticks down one", () => {
				test("Stays as 1:00 when time is rounded to 1 minute, not 60 secs", () => {
					expect(formatTimeToClock(59999, ["s", "d"], true)).toEqual("1:00");
				});

				test("Shows 59 seconds when time is exactly 59s", () => {
					expect(formatTimeToClock(59000, ["s", "d"], true)).toEqual("59");
				});

				test("Goes to 59.999 sec when ms are enabled", () => {
					expect(formatTimeToClock(59999, ["ms", "d"], true)).toEqual("59.999");
				});

				test("Does not use units larger than the given unit range", () => {
					expect(formatTimeToClock(3_700_000, ["s", "m"])).toEqual("61:40");
				});
			});
		});
	});
});
