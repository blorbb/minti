import {
	convert,
	order,
	unitStrings,
	type TimeAbbreviations,
} from "./timer_utils";

class TokenManager {
	public static readonly LETTER_TOKEN_REGEX = /[a-z]/i;
	public static readonly NUMBER_TOKEN_REGEX = /[-0-9.]/i;
	public static readonly SEPARATOR = ":";

	public inputChars;
	constructor(public readonly input: string) {
		this.inputChars = input.split("");
	}

	// first input token is always going to be ""
	public inputTokens: { string: string; type: TokenTypes }[] = [];
	public prevTokenType: TokenTypes = "unknown";
	private currentToken = "";
	public pushCurrentToken() {
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
		if (char === TokenManager.SEPARATOR) return "separator";
		else if (TokenManager.LETTER_TOKEN_REGEX.test(char)) return "letter";
		else if (TokenManager.NUMBER_TOKEN_REGEX.test(char)) return "number";
		else return "unknown";
	}
}

export function parseInput(input: string) {
	const tokenList = parseInputTokens(input);

	// check that everything is valid
	// and the input uses EITHER letters or separators
	let parsingMethod: "" | "separator" | "letter" = "";
	let separatorCount = 0;
	for (const token of tokenList) {
		// numbers must be valid
		if (token.type === "number") {
			const number = +token.string;
			if (isNaN(number)) throw new Error("Invalid number");
			continue;
		}
		// should have already been checked before
		// but just in case
		/* c8 ignore next 3 */
		if (token.type === "unknown") throw new Error("Invalid input");

		// tracking stuff
		if (token.type === "separator") separatorCount++;
		else if (!unitStrings.VALID_STRINGS.includes(token.string))
			throw new Error("Invalid units");

		if (parsingMethod) {
			// method is already defined
			// also count the number of separators
			// same method
			if (token.type === parsingMethod) continue;
			// different method: invalid
			else throw new Error("Cannot have multiple methods");
		}
		// new method
		parsingMethod = token.type;
	}

	// checks:
	// tokenList only uses one method (separators or letters)
	// and all number-strings can be cast to number
	// and all the letter-strings are valid units

	// maximum of 3 separators
	// for 12:34:56:59.000
	//     d  h  m  s
	if (separatorCount > 3) throw new Error('Too many separators ":"');

	// parsingMethod is either:
	// "" for a plain number (also means there is only one token)
	// "separator" for *:*:* format
	// "letter" for *h*m*s format
	if (parsingMethod === "") {
		return convert.minsToMs(+tokenList[0].string);
	}

	let totalTime = 0;
	let currentNumber = 0;
	let currentUnit: TimeAbbreviations | null = null;
	for (const token of tokenList) {
		switch (token.type) {
			case "number": {
				currentNumber = +token.string;
				break;
			}
			case "letter": {
				const unit = unitStrings.stringToUnit(token.string);
				const time = convert.timeUnitToMs(currentNumber, unit);
				totalTime += time;
				currentNumber = 0;
				currentUnit = unit;
				break;
			}
			case "separator": {
				// convert count to a unit
				// e.g. 12:34 => 1 sep => minutes (index 2)
				// 35:45:12 => 2 sep => hours (index 3)
				// already guaranteed that the index is in range
				// from prev check (separator > 3)
				const unit = order.INDEX_TO_UNITS[separatorCount + 1];
				const time = convert.timeUnitToMs(currentNumber, unit);
				separatorCount--;
				totalTime += time;
				currentNumber = 0;
				currentUnit = unit;
				break;
			}
		}
	}
	if (currentNumber !== 0 && currentUnit) {
		if (currentUnit === "ms")
			throw new Error("No units smaller than ms accepted");

		const currentUnitIndex = order.UNITS_TO_INDEX[currentUnit];
		const nextUnit = order.INDEX_TO_UNITS[currentUnitIndex - 1];
		totalTime += convert.timeUnitToMs(currentNumber, nextUnit);
	}

	return totalTime;
}

type TokenTypes = "letter" | "number" | "separator" | "unknown";
function parseInputTokens(input: string) {
	const manager = new TokenManager(input);

	for (const char of manager.inputChars) {
		const tokenType = TokenManager.charToTokenType(char);

		// invalid input
		if (tokenType === "unknown") {
			throw new Error("Invalid input");
		}

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
