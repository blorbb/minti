import { convert, parseFromTime } from "./timer_utils";
import { describe, test, expect } from "vitest";

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
			expect(parseFromTime.toUnits(time)).toEqual({ d, h, m, s, ms });
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
				expect(parseFromTime.toUnits(time)).toEqual({ d, h, m, s, ms });
			},
		);
	});

	describe("From milliseconds to a clock time", () => {
		describe("Handles trims and padding", () => {
			test("Produces correct time range", () => {
				expect(parseFromTime.toClock(137_020, ["ms", "m"])).toEqual("2:17.020");
			});

			test("Can swap the given time range", () => {
				expect(parseFromTime.toClock(137_020, ["m", "ms"])).toEqual("2:17.020");
			});

			test("Can convert large units into the given range", () => {
				expect(parseFromTime.toClock(5_315_938, ["ms", "m"])).toEqual(
					"88:35.938",
				);
			});

			test("Can allow units to have a longer string when LHS is trimmed", () => {
				expect(parseFromTime.toClock(325_225, ["ms", "s"])).toEqual("325.225");
			});

			test("Adds necessary 0 padding", () => {
				expect(parseFromTime.toClock(123_456, ["ms", "h"])).toEqual(
					"0:02:03.456",
				);
			});

			test("Provides a default time range", () => {
				expect(parseFromTime.toClock(2040)).toEqual("0:00:00:02.040");
			});

			test("Adds 1 to rightmost unit when positive", () => {
				expect(parseFromTime.toClock(137_020, ["s", "m"])).toEqual("2:18");
			});

			test("Does not add 1 when RHS unit is 0", () => {
				expect(parseFromTime.toClock(137_000, ["s", "m"])).toEqual("2:17");
			});

			test("Supports single units, not adding separators", () => {
				expect(parseFromTime.toClock(3_242_521, ["s", "s"])).toEqual("3243");
			});

			test("Can trim down to ms", () => {
				expect(parseFromTime.toClock(100_000_000, ["ms", "ms"])).toEqual(
					"100000000",
				);
			});

			test("Can trim to days only", () => {
				expect(parseFromTime.toClock(200_000_000, ["d", "d"])).toEqual("3");
			});

			test("Number of days is 2 (not 3) when time is exactly 2 days", () => {
				expect(parseFromTime.toClock(172_800_000, ["d", "d"])).toEqual("2");
			});
		});

		describe("Handles negative times", () => {
			test("Adds minus sign when negative", () => {
				expect(parseFromTime.toClock(-19_394)).toEqual("-0:00:00:19.394");
			});

			test("Shifts minus sign when LHS is trimmed", () => {
				expect(parseFromTime.toClock(-38_471, ["ms", "m"])).toEqual(
					"-0:38.471",
				);
			});

			test("When trimmed time is 0, still recognises as negative", () => {
				expect(parseFromTime.toClock(-230, ["s", "m"])).toEqual("-0:00");
			});

			test("Does not add 1 to time when negative", () => {
				expect(parseFromTime.toClock(-23_423, ["s", "m"])).toEqual("-0:23");
			});
		});

		describe("Handles auto mode", () => {
			test.each<[number, string]>([
				[100_000_000, "1:03:46:40.000"],
				[5_324_542, "1:28:44.542"],
				[847_231, "14:07.231"],
				[3742, "3.742"],
			])("Auto-trims each of the units: from %ims to %s", (time, result) => {
				expect(parseFromTime.toClock(time, ["ms", "d"], true)).toEqual(result);
			});

			test("Keeps 0 seconds when in the last second of the timer", () => {
				expect(parseFromTime.toClock(123, ["d", "ms"], true)).toEqual("0.123");
			});

			test("Can auto-trim to a restricted set of units", () => {
				expect(parseFromTime.toClock(500_000, ["ms", "s"], true)).toEqual(
					"500.000",
				);
			});

			test("Can auto-trim from the right side", () => {
				expect(parseFromTime.toClock(653_838, ["s", "d"], true)).toEqual(
					"10:54",
				);
			});
		});
	});
});

describe("Converts", () => {
	// temporary to cover a switch case
	// TODO refactor all these tests, better describes and names
	test("From ms to days", () => {
		expect(convert.msToTimeUnit(43_200_000, "d")).toEqual(0.5);
	});
});
