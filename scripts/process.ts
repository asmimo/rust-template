import cp from "node:child_process";

import type { ReadonlyDeep } from "type-fest";

const resolvedCode = 0;
const exitCode = 1;

// oxlint-disable-next-line typescript/promise-function-async
export const spawnSafe = (
	program: string,
	args: readonly string[],
	options: ReadonlyDeep<{ cwd?: string; env?: NodeJS.ProcessEnv }> = {},
): Promise<void> =>
	new Promise<void>((resolve, reject) => {
		let interrupted = false;

		const child = cp.spawn(program, args, {
			shell: false,
			stdio: "inherit",
			...options,
		});

		const onSigint = (): void => {
			interrupted = true;
		};
		process.once("SIGINT", onSigint);

		child.on("close", (code) => {
			process.off("SIGINT", onSigint);
			if (code === resolvedCode || interrupted) {
				resolve();
			} else {
				reject(new Error(`Command failed with exit code ${code}`));
			}
		});

		child.on("error", (error: Readonly<Error>) => {
			process.off("SIGINT", onSigint);
			reject(error);
		});
	});

export const catchError = (error: unknown): void => {
	if (error instanceof Error) {
		if (error.name === "ExitPromptError") {
			console.log("\nGracefully shutting down from SIGINT (Ctrl-C)");
		} else {
			console.error("An error occurred:", error.message);
			if (error.stack !== undefined) {
				console.error(error.stack);
			}
			process.exit(exitCode);
		}
	} else {
		console.error("An unexpected error occurred:", error);
		process.exit(exitCode);
	}
};
