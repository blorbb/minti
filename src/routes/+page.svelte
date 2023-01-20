<script lang="ts">
	import { Timer } from "$lib/utils/timer";

	const timer = new Timer(5000);
	let timerTime = 5000;
	let stringTime = "";
	let ended = false;

	timer.onTimeChange((t) => {
		timerTime = t;
		stringTime = Timer.parseToClock(t, ["m", "ms"]).slice(0, -1);
	});

	timer.onTimerEnd(() => {
		ended = true;
	});
</script>

<h2>
	ms counter: {timerTime}
</h2>
<h2 style:color={ended ? "red" : ""}>
	clock counter: {stringTime}
</h2>

<button on:click={timer.start}>start</button>
<button on:click={timer.reset}>reset</button>
<button on:click={timer.resume}>resume</button>
<button on:click={timer.pause}>pause</button>
