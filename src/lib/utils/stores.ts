import { get, type Readable, writable } from "svelte/store";
import { sleep } from "./misc";
import { getCSSProp } from "./css";
import { TimerController } from "./timer_controller";
import type { UnitRange } from "./timer_utils";

//#region timer list
interface TimerControllerListStore extends Readable<TimerController[]> {
	removeTimer: (timer: TimerController) => void;
	addTimer: () => void;
	removeAll: () => void;
}

function initTimerControllerList(): TimerControllerListStore {
	const store = writable([new TimerController()]);

	async function removeTimer(timer: TimerController) {
		store.update((list) => list.filter((t) => !Object.is(t, timer)));
		// make sure there's always one timer
		if (get(store).length === 0) {
			// wait for it to disappear first
			await sleep(getCSSProp("--t-transition", "time") ?? 100);
			store.set([new TimerController()]);
		}
	}

	function addTimer() {
		store.update((list) => [...list, new TimerController()]);
	}

	async function removeAll() {
		store.set([]);
		await sleep(getCSSProp("--t-transition", "time") ?? 100);
		store.set([new TimerController()]);
	}

	return {
		subscribe: store.subscribe,
		removeTimer,
		addTimer,
		removeAll,
	};
}

export const timerControllerList = initTimerControllerList();
//#endregion

// settings

type Settings = {
	timerUpdateInterval: number;
	autoTrimTimerDisplay: boolean;
	timerUnitRange: UnitRange;
	progressBarType: "line" | "background";
	progressBarBackgroundBorder: number;
	buttonScaleDuration: number;
};

export const settings = writable<Settings>({
	timerUpdateInterval: 200,
	autoTrimTimerDisplay: true,
	timerUnitRange: ["s", "d"],
	progressBarType: "background",
	progressBarBackgroundBorder: 1,
	buttonScaleDuration: 200,
});
