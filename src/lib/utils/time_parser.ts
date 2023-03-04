/**
 * function to convert a user inputted time (string) to a time in ms
 *
 * TODO: date and times ("3:30pm", "tmr 3:20pm", somehow a 24h clock without being ambiguous to mm:ss?)
 */

import {
	convert,
	order,
	unitStrings,
	type TimeAbbreviations,
} from "./timer_utils";

export class ParseError extends Error {
	constructor(message: string) {
		super(message);
		this.name = "ParseError";
	}
}

export function parseInput(input: string) {
	const tokenList = parseInputTokens(input);
	const listInfo = validateTokenList(tokenList);

	// checks: see @returns of `validateTokenList`

	// index of the time unit to convert a number to ms
	// used by order.INDEX_TO_UNITS
	// index of day = 4
	// d:hh:mm:ss => 3 separators => day unit => index 4
	let currentSeparatorUnitIndex = listInfo.separatorCount + 1;

	if (listInfo.parsingMethod === "singleNumber") {
		return convert.minsToMs(+tokenList[0].string);
	}

	let totalTime = 0;
	/**
	 * When token.type = "letter" or "separator", `currentNumber`
	 * is the number that preceded that letter/separator.
	 */
	let currentNumber = 0;
	/**
	 * Needed to keep track of the unit for inferring the
	 * unit of the last number, if token.type = "letter".
	 * e.g. 1m30 => 1 minute 30 seconds.
	 * `currentNumber` is only added to `totalTime` when
	 * a separator or letter is present, so the final number
	 * (e.g. 2:**20**) is not added.
	 */
	let currentUnit: TimeAbbreviations | null = null;
	for (const token of tokenList) {
		switch (token.type) {
			case "number": {
				currentNumber = +token.string;
				break;
			}
			case "letter": {
				const timeUnit = unitStrings.stringToUnit(token.string);
				const timeToAdd = convert.timeUnitToMs(currentNumber, timeUnit);

				totalTime += timeToAdd;
				currentNumber = 0;
				currentUnit = timeUnit;

				break;
			}
			case "separator": {
				// convert count to a unit
				// already guaranteed that the index is in range
				// from prev check (separator > 3)
				const timeUnit = order.INDEX_TO_UNITS[currentSeparatorUnitIndex];
				const timeToAdd = convert.timeUnitToMs(currentNumber, timeUnit);

				// next unit is 1 lower
				currentSeparatorUnitIndex--;

				totalTime += timeToAdd;
				currentNumber = 0;
				currentUnit = timeUnit;

				break;
			}
		}
	}

	// final number
	const hasUnusedNumberToAdd = currentNumber !== 0 && currentUnit !== null;

	if (hasUnusedNumberToAdd) {
		// throw error for strings like "10ms20"
		// the "20" after "ms" has no valid units
		if (currentUnit === "ms")
			throw new ParseError("No units smaller than ms accepted");

		// can't use `currentSeparatorUnitIndex` as this applies for
		// both separators and letters
		// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
		const currentUnitIndex = order.UNITS_TO_INDEX[currentUnit!];
		const nextSmallerUnit = order.INDEX_TO_UNITS[currentUnitIndex - 1];

		totalTime += convert.timeUnitToMs(currentNumber, nextSmallerUnit);
	}

	return totalTime;
}

type TokenTypes = "letter" | "number" | "separator";
type TokenInfo = {
	type: TokenTypes;
	string: string;
};

/**
 * Helper class to convert a string to a token list.
 *
 * Used by `parseInputTokens` only.
 */
class TokenManager {
	public static readonly LETTER_TOKEN_REGEX = /[a-z]/i;
	public static readonly NUMBER_TOKEN_REGEX = /[-0-9.]/i;

	public inputChars;
	constructor(public readonly input: string) {
		this.inputChars = input.split("");
	}

	// first input token is always going to be ""
	public inputTokens: { string: string; type: TokenTypes }[] = [];
	public prevTokenType: TokenTypes | null = null;
	private currentToken = "";

	public pushCurrentToken() {
		// prevTokenType is only null on the first iteration
		// first iteration won't be used anyway as the token will be ""
		if (this.prevTokenType === null) this.prevTokenType = "number";

		this.inputTokens.push({
			string: this.currentToken,
			type: this.prevTokenType,
		});
		this.currentToken = "";
	}

	public appendToCurrentToken(char: string) {
		this.currentToken += char;
	}

	public static charToTokenType(char: string): TokenTypes {
		if (char === unitStrings.UNIT_SEPARATOR) return "separator";
		else if (TokenManager.LETTER_TOKEN_REGEX.test(char)) return "letter";
		else if (TokenManager.NUMBER_TOKEN_REGEX.test(char)) return "number";
		else throw new ParseError("Invalid input");
	}
}

function parseInputTokens(input: string) {
	if (input === "") throw new ParseError("Please enter a time");

	const noWhitespaceInput = input.replace(/\s/gm, "");
	const manager = new TokenManager(noWhitespaceInput);

	for (const char of manager.inputChars) {
		const tokenType = TokenManager.charToTokenType(char);

		if (tokenType === "separator") {
			// push currentToken, even if both types are the same
			// allows for separators without numbers
			// e.g. 5::2 = 5hr 2sec
			manager.pushCurrentToken();
			manager.appendToCurrentToken(char);
			manager.prevTokenType = tokenType;
			continue;
		}

		// add to current token
		if (tokenType === manager.prevTokenType) {
			manager.appendToCurrentToken(char);
			continue;
		}

		// push current token
		manager.pushCurrentToken();
		manager.appendToCurrentToken(char);
		manager.prevTokenType = tokenType;
	}

	// add the last bit
	manager.pushCurrentToken();

	// first token is "" because currentToken is initially "" and pushed
	return manager.inputTokens.slice(1);
}

/**
 * the format of the string:
 * - "singleNumber": the string is just a number, e.g. "3.4"
 * - "separator": string is separated by ":" (or something else,
 * depending on config) e.g. "1:30:00"
 * - "letter": each number has an associated unit, e.g. 1h30m
 */
type ParsingMethod = "singleNumber" | "separator" | "letter";
type ParsedListInfo = {
	parsingMethod: ParsingMethod;
	/**
	 * number of separators (":") in the token list
	 *
	 * only if parsingMethod = "separator"
	 *
	 * must be <= 3 (days **:** hours **:** minutes **:** seconds)
	 */
	separatorCount: number;
};

/**
 *
 * @param tokenList Token list generated by `parseInputTokens`
 * @returns Information about the token list: checks that
 *
 * All numbers used are valid numbers
 *
 * There is only one method used to denote units:
 * - A number (`"3.4"` => 3.4 minutes)
 * - Separators (default `":"`) (`"1:30"` => 1 minute 30 sec)
 * - Letters (`"2h10m"` => 2 hours 10 minutes)
 *
 * If the method is `"separator"`, also checks that there are a
 * maximum of 3 separators (up to `d:hh:mm:ss`, any more is ambiguous).
 * Milliseconds aren't a separator, just a decimal after seconds (`secs.ms`).
 *
 * If any of these conditions are invalid, function will throw.
 */
function validateTokenList(tokenList: TokenInfo[]) {
	const listInfo: ParsedListInfo = {
		parsingMethod: "singleNumber",
		separatorCount: 0,
	};

	// check that the input uses EITHER letters or separators
	// and all numbers are valid
	for (const token of tokenList) {
		// numbers must be valid
		if (token.type === "number") {
			const number = +token.string;
			if (isNaN(number)) throw new ParseError("Invalid number");
			continue;
		}

		// token type is either "separator" or "letter"

		// established parsing method already
		// separator: track separatorCount
		if (token.type === "separator") listInfo.separatorCount++;
		// letters: ensure the units are valid
		else if (!unitStrings.VALID_STRINGS.includes(token.string))
			throw new ParseError("Invalid units");

		if (listInfo.parsingMethod !== "singleNumber") {
			// parsing method is already defined
			// ensure that its the same parsing method
			if (token.type === listInfo.parsingMethod) continue;
			// different method: invalid
			else throw new ParseError("Cannot have multiple methods");
		}

		// haven't established parsing method yet
		// set the method
		listInfo.parsingMethod = token.type;
	}

	// ensure that there aren't too many separators

	// maximum of 3 separators
	// for 12:34:56:59.000
	//     d  h  m  s
	if (listInfo.separatorCount > 3)
		throw new ParseError(`Too many separators "${unitStrings.UNIT_SEPARATOR}"`);

	return listInfo;
}
