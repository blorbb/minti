export class Signal {
	private receivers: SignalReceiver[] = [];

	public newReceiver() {
		const recv = new SignalReceiver(this);
		this.receivers.push(recv);
		return recv;
	}

	public emit(...args: unknown[]) {
		this.receivers.forEach((r) => r.activate(...args));
	}

	public deleteReceiver(target: SignalReceiver) {
		this.receivers = this.receivers.filter((recv) => Object.is(recv, target));
	}
}

export class SignalReceiver {
	constructor(public readonly emitter: Signal) {}

	private callback?: (...args: unknown[]) => void;
	public onSignal(fn: typeof this.callback) {
		this.callback = fn;
	}

	public activate(...args: unknown[]) {
		if (this.callback) this.callback(...args);
	}

	public delete() {
		this.emitter.deleteReceiver(this);
	}
}
