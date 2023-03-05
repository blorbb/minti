import { differenceInDays, toLocalISODate, toTime } from "$lib/utils/date";
import { describe, test, expect } from "vitest";

describe("Date", () => {
	describe("Formatting", () => {
		describe("To ISO Date", () => {
			test("Is based on local time", () => {
				expect(toLocalISODate(new Date(2023, 4, 3))).toEqual("2023-05-03");
			});

			test("Excludes the time", () => {
				expect(toLocalISODate(new Date(Date.UTC(2023, 5, 6, 0, 0)))).toEqual(
					"2023-06-06",
				);
			});
		});

		describe("12 hour time", () => {
			test("Converts post-noon 13:05 to 12 hour time 1:05 pm", () => {
				expect(toTime(new Date(2023, 4, 5, 13, 5), "12h")).toEqual("1:05 pm");
			});

			test("Converts pre-noon 7:23 to 12 hour time 7:23 am", () => {
				expect(toTime(new Date(2023, 1, 12, 7, 23), "12h")).toEqual("7:23 am");
			});

			test("Converts midnight to 12:00 am", () => {
				expect(toTime(new Date(2019, 3, 16, 0, 0, 0, 0), "12h")).toEqual(
					"12:00 am",
				);
			});

			test("Converts noon to 12:00 pm", () => {
				expect(toTime(new Date(1999, 8, 26, 12, 0, 0, 0), "12h")).toEqual(
					"12:00 pm",
				);
			});
		});

		describe("24 hour time", () => {
			test("Keeps post-noon time 22:02 in 24h time", () => {
				expect(toTime(new Date(2022, 1, 2, 22, 2), "24h")).toEqual("22:02");
			});

			test("Keeps pre-noon time 7:12 in 24h time with padding", () => {
				expect(toTime(new Date(2010, 6, 4, 7, 12), "24h")).toEqual("07:12");
			});

			test("Converts midnight to 00:00", () => {
				expect(toTime(new Date(2019, 3, 16, 0, 0, 0, 0), "24h")).toEqual(
					"00:00",
				);
			});

			test("Converts noon to 12:00", () => {
				expect(toTime(new Date(1999, 8, 26, 12, 0, 0, 0), "24h")).toEqual(
					"12:00",
				);
			});
		});
	});

	describe("Day difference", () => {
		test("0 days difference two dates on the same day", () => {
			expect(
				differenceInDays(new Date(2000, 4, 4, 1), new Date(2000, 4, 4, 23)),
			).toEqual(0);
		});
		test("1 day difference two dates if it crosses midnight", () => {
			expect(
				differenceInDays(
					new Date(2000, 4, 4, 23, 59),
					new Date(2000, 4, 5, 0, 1),
				),
			).toEqual(1);
		});
	});
});
