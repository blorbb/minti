<script lang="ts">
	import { Timer } from "$lib/utils/timer";
	import { onDestroy, onMount } from "svelte";

	const timer = new Timer(5000);
	let timerTime = 5000;
	let stringTime = "";
	let ended = false;

	$: {
		stringTime = Timer.parseToClock(timerTime);
		console.log({ timerTime });
	}

	// using interval: NodeJS.Timer raises a linting error
	let interval: ReturnType<typeof setInterval>;

	onMount(() => {
		interval = setInterval(() => {
			timerTime = timer.getTimeRemaining();
			console.log("timerTime", timerTime);
			console.log("timeremaining", timer.getTimeRemaining());
		}, 10);
	});

	onDestroy(() => {
		clearInterval(interval);
	});
</script>

<h2>
	ms counter: {timerTime}
</h2>
<h2 style:color={ended ? "red" : ""}>
	clock counter: {stringTime}
</h2>

<button on:click={timer.start}>start</button>
<button on:click={() => timer.reset()}>reset</button>
<button on:click={timer.resume}>resume</button>
<button on:click={timer.pause}>pause</button>
