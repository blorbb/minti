import { Signal } from "$lib/utils/signal";
import { describe, test, expect, vi } from "vitest";

describe("Signal", () => {
	test("Receiver can receive a signal", () => {
		const mock = vi.fn();
		const signal = new Signal();
		signal.newReceiver().onSignal(mock);

		signal.emit();

		expect(mock).toHaveBeenCalledOnce();
	});

	test("Can have multiple receivers", () => {
		const mock1 = vi.fn();
		const mock2 = vi.fn();

		const signal = new Signal();

		const receiver1 = signal.newReceiver();
		const receiver2 = signal.newReceiver();
		receiver1.onSignal(mock1);
		receiver2.onSignal(mock2);

		signal.emit();

		expect(mock1).toHaveBeenCalledOnce();
		expect(mock2).toHaveBeenCalledOnce();
	});

	test("Receiver can take arguments", () => {
		const mock1 = vi.fn();
		const mock2 = vi.fn();

		const signal = new Signal();

		const receiver = signal.newReceiver();
		let counter = 0;
		receiver.onSignal((id, increment) => {
			if (id === "1") mock1();
			else mock2();
			if (typeof increment === "number") counter += increment;
		});

		signal.emit("1", 20);
		signal.emit(1234, 1);
		signal.emit("2", 5);

		expect(mock1).toHaveBeenCalledOnce();
		expect(mock2).toHaveBeenCalledTimes(2);
		expect(counter).toEqual(26);
	});

	test("Can remove receivers", () => {
		const mock = vi.fn();

		const signal = new Signal();
		const receiver1 = signal.newReceiver();
		const receiver2 = signal.newReceiver();

		receiver1.onSignal(mock);
		receiver2.onSignal(mock);

		signal.emit(); // 2 calls of mock
		receiver1.delete();
		signal.emit(); // 1 more call

		expect(mock).toHaveBeenCalledTimes(3);
	});
});
