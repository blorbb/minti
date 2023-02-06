import { TimerController, type TimeAbbreviations } from "./timer_controller";

class TokenManager {
	public static readonly LETTER_TOKEN_REGEX = /[a-z]/i;
	public static readonly NUMBER_TOKEN_REGEX = /[-0-9.]/i;
	public static readonly SEPARATOR = ":";

	public inputChars;
	constructor(public readonly input: string) {
		this.inputChars = input.split("");
	}

	// first input token is always going to be ""
	public inputTokens: { token: string; type: TokenTypes }[] = [];
	public prevTokenType: TokenTypes = "unknown";
	private currentToken = "";
	public pushCurrentToken() {
		this.inputTokens.push({
			token: this.currentToken,
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

parseInput("1");

export function parseInput(input: string) {
	const tokenList = parseInputTokens(input);

	// check that everything is valid
	// and the input uses EITHER letters or separators
	let parsingMethod: "" | "separator" | "letter" = "";
	let separatorCount = 0;
	for (const token of tokenList) {
		// numbers must be valid
		if (token.type === "number") {
			const number = +token.token;
			if (isNaN(number)) throw new Error("Invalid number");
			continue;
		}
		// should have already been checked before
		// but just in case
		if (token.type === "unknown") throw new Error("Invalid input");

		// tracking stuff
		if (token.type === "separator") separatorCount++;
		else if (!VALID_STRINGS.includes(token.token))
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
		return TimerController.MS_IN_MIN * +tokenList[0].token;
	}

	let totalTime = 0;
	let currentNumber = 0;
	let currentUnit: TimeAbbreviations | null = null;
	for (const token of tokenList) {
		switch (token.type) {
			case "number": {
				currentNumber = +token.token;
				break;
			}
			case "letter": {
				const unit = stringTokenToUnit(token.token);
				const time = unitTimeToMs(currentNumber, unit);
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
				const unit = TimerController.INDEX_TO_UNITS[separatorCount + 1];
				separatorCount--;
				const time = unitTimeToMs(currentNumber, unit);
				totalTime += time;
				currentNumber = 0;
				currentUnit = unit;
				break;
			}
		}
	}
	if (currentNumber !== 0 && currentUnit) {
		const nextUnitIndex = TimerController.UNITS_TO_INDEX[currentUnit];
		if (nextUnitIndex === 0)
			throw new Error("No units smaller than ms accepted");
		const nextUnit = TimerController.INDEX_TO_UNITS[nextUnitIndex - 1];
		totalTime += unitTimeToMs(currentNumber, nextUnit);
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

const DAY_STRINGS = ["d", "day", "days"];
const HOUR_STRINGS = ["h", "hr", "hrs", "hour", "hours"];
const MIN_STRINGS = ["m", "min", "mins", "minute", "minutes"];
const SEC_STRINGS = ["s", "sec", "secs", "second", "seconds"];
const MS_STRINGS = [
	"ms",
	"milli",
	"millis",
	"millisec",
	"millisecs",
	"millisecond",
	"milliseconds",
];

const VALID_STRINGS = [
	...DAY_STRINGS,
	...HOUR_STRINGS,
	...MIN_STRINGS,
	...SEC_STRINGS,
	...MS_STRINGS,
];

function stringTokenToUnit(token: string): TimeAbbreviations {
	if (DAY_STRINGS.includes(token)) return "d";
	else if (HOUR_STRINGS.includes(token)) return "h";
	else if (MIN_STRINGS.includes(token)) return "m";
	else if (SEC_STRINGS.includes(token)) return "s";
	else if (MS_STRINGS.includes(token)) return "ms";
	else throw new Error("Invalid unit");
}

function unitTimeToMs(num: number, unit: TimeAbbreviations) {
	if (unit === "d") return num * TimerController.MS_IN_DAY;
	else if (unit === "h") return num * TimerController.MS_IN_HOUR;
	else if (unit === "m") return num * TimerController.MS_IN_MIN;
	else if (unit === "s") return num * TimerController.MS_IN_SEC;
	else return num;
}
