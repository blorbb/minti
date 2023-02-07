import { describe, expect, test } from "vitest";
import { convert } from "./timer_utils";
import { parseInput as p } from "./time_parser";

// quick functions to easily convert units to ms
const d = convert.daysToMs;
const h = convert.hoursToMs;
const m = convert.minsToMs;
const s = convert.secsToMs;
function ms(ms: number) {
	return ms;
}

describe("Rejects", () => {
	test("Having both separators and letters", () => {
		expect(() => p("1:34h")).toThrow();
	});

	test("Words that are not valid units", () => {
		expect(() => p("1ha")).toThrow();
	});

	test.each(["aa", "oaik", "an hour", "a bunch of text"])(
		"Random words",
		(input) => {
			expect(() => p(input)).toThrow();
		},
	);

	test("Invalid numbers", () => {
		expect(() => p("3-5")).toThrow();
	});

	test("Too many separators", () => {
		expect(() => p("1:2:3:4:5")).toThrow();
	});

	test("Cannot infer after milliseconds", () => {
		expect(() => p("83ms24")).toThrow();
	});
});

describe("Parses a duration", () => {
	describe("Plain numbers", () => {
		test("Converts a number to minutes", () => {
			expect(p("12")).toEqual(m(12));
		});

		test("Converts a number greater than 60", () => {
			expect(p("893")).toEqual(m(893));
		});

		describe("With decimals", () => {
			test("Converts 3.5 minutes to 3 min 30 sec", () => {
				expect(p("3.5")).toEqual(m(3.5));
			});

			test("Allows decimals without leading 0", () => {
				expect(p(".4")).toEqual(m(0.4));
			});
		});
	});

	describe("Clock time", () => {
		describe("One separator", () => {
			test("Converts *:** to minutes and seconds", () => {
				expect(p("1:34")).toEqual(m(1) + s(34));
			});

			test("Can convert large numbers for minutes (***:**)", () => {
				expect(p("431:15")).toEqual(m(431) + s(15));
			});

			test("Allows for seconds > 60", () => {
				expect(p("15:90")).toEqual(m(15) + s(90));
			});

			test("Allows for many digit seconds", () => {
				expect(p("3:8395")).toEqual(m(3) + s(8395));
			});

			test("Allows for single digit seconds", () => {
				expect(p("5:2")).toEqual(m(5) + s(2));
			});
		});

		describe("Multiple separators", () => {
			test("Converts *:**:** to hours, minutes and seconds", () => {
				expect(p("6:23:34")).toEqual(h(6) + m(23) + s(34));
			});

			test("Can convert large numbers for hours (***:**:**)", () => {
				expect(p("3841:34:51")).toEqual(h(3841) + m(34) + s(51));
			});

			test("Converts *:**:**:** to days, hours, minutes and seconds", () => {
				expect(p("5:22:54:03")).toEqual(d(5) + h(22) + m(54) + s(3));
			});

			test("Allows single digit units", () => {
				expect(p("3:5:7:1")).toEqual(d(3) + h(5) + m(7) + s(1));
			});

			test("Allows overflowing units", () => {
				expect(p("93:54:99:81")).toEqual(d(93) + h(54) + m(99) + s(81));
			});

			test("Allows for separators without numbers", () => {
				expect(p("5::2")).toEqual(h(5) + s(2));
			});
		});

		describe("With decimals", () => {
			test("Uses milliseconds after dot", () => {
				expect(p("5:13.273")).toEqual(m(5) + s(13) + ms(273));
			});

			test("Accepts decimals not at the end as well", () => {
				expect(p("43:3.5:23")).toEqual(h(43) + m(3.5) + s(23));
			});

			test("Accepts multiple decimals", () => {
				expect(p("34.2:1.74:23.45432")).toEqual(
					h(34.2) + m(1.74) + s(23.45432),
				);
			});

			test("Accepts decimal without leading 0", () => {
				expect(p(".3:3")).toEqual(m(0.3) + s(3));
			});
		});
	});

	describe("With Letters", () => {
		describe("Single unit", () => {
			test.each(["d", "day", "days"])("Accepts day units: 84%s", (unit) => {
				expect(p(`84${unit}`)).toEqual(d(84));
			});

			test.each(["h", "hr", "hrs", "hour", "hours"])(
				"Accepts hour units: 34%s",
				(unit) => {
					expect(p(`34${unit}`)).toEqual(h(34));
				},
			);

			test.each(["m", "min", "mins", "minute", "minutes"])(
				"Accepts minute units: 56%s",
				(unit) => {
					expect(p(`56${unit}`)).toEqual(m(56));
				},
			);

			test.each(["s", "sec", "secs", "second", "seconds"])(
				"Accepts second units: 143%s",
				(unit) => {
					expect(p(`143${unit}`)).toEqual(s(143));
				},
			);

			test.each([
				"ms",
				"milli",
				"millis",
				"millisec",
				"millisecs",
				"millisecond",
				"milliseconds",
			])("Accepts millisecond units: 8739147%s", (unit) => {
				expect(p(`8739147${unit}`)).toEqual(ms(8739147));
			});
		});

		describe("Multiple units", () => {
			test.each([
				["1s43ms", s(1) + ms(43)],
				["1m43ms", m(1) + ms(43)],
				["1h43ms", h(1) + ms(43)],
				["1d43ms", d(1) + ms(43)],
				["1m43s", m(1) + s(43)],
				["1h43s", h(1) + s(43)],
				["1d43s", d(1) + s(43)],
				["1h43m", h(1) + m(43)],
				["1d43m", d(1) + m(43)],
				["1d43h", d(1) + h(43)],
			])("Converts 2 units %s to %i ms", (input, time) => {
				expect(p(input)).toEqual(time);
			});

			test.each([
				["5ms12s", ms(5) + s(12)],
				["5ms12m", ms(5) + m(12)],
				["5ms12h", ms(5) + h(12)],
				["5ms12d", ms(5) + d(12)],
				["5s12m", s(5) + m(12)],
				["5s12h", s(5) + h(12)],
				["5s12d", s(5) + d(12)],
				["5m12h", m(5) + h(12)],
				["5m12d", m(5) + d(12)],
				["5h12d", h(5) + d(12)],
			])("Converts 2 out of order units %s to %i ms", (input, time) => {
				expect(p(input)).toEqual(time);
			});

			test.each([
				["3h13m5s", h(3) + m(13) + s(5)],
				["1d93h2s", d(1) + h(93) + s(2)],
				["3d1m9ms", d(3) + m(1) + ms(9)],
				["2h45m34ms", h(2) + m(45) + ms(34)],
				["1d93h3m2s543ms", d(1) + h(93) + m(3) + s(2) + ms(543)],
			])("Converts many units %s to %i ms", (input, time) => {
				expect(p(input)).toEqual(time);
			});

			describe("With decimals", () => {
				test("Allows decimals in units", () => {
					expect(p("32.1h65.2m2s")).toEqual(h(32.1) + m(65.2) + s(2));
				});

				test("Allows decimals without leading 0", () => {
					expect(p(".5h3s")).toEqual(h(0.5) + s(3));
				});
			});
		});

		describe("Infers units", () => {
			test.each([
				["1m43", m(1) + s(43)],
				["934d47", d(934) + h(47)],
				["1d3m54", d(1) + m(3) + s(54)],
			])(
				"Assumes number after unit is the lower unit: %s to %ims",
				(input, time) => {
					expect(p(input)).toEqual(time);
				},
			);

			describe("With decimals", () => {
				test("Allows decimals", () => {
					expect(p("84.1m1.8")).toEqual(m(84.1) + s(1.8));
				});

				test("Allows decimals without leading 0", () => {
					expect(p(".6m.2s")).toEqual(m(0.6) + s(0.2));
				});
			});
		});
	});
});

// TO ADD: date and time timers
