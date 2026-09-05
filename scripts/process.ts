import cp from "node:child_process";
import { promisify } from "node:util";

export const exec = promisify(cp.exec);

export const runCommand = async (cmd: string, args: string[] = []) => {
	return new Promise<void>((resolve, reject) => {
		const child = cp.spawn(cmd, args, {
			stdio: "inherit",
			shell: true,
		});

		child.on("close", (code) => {
			if (code === 0) {
				resolve();
			} else {
				reject(new Error(`Command failed with exit code ${code}`));
			}
		});

		child.on("error", (error) => {
			reject(error);
		});
	});
};

export const catchError = (error: unknown) => {
	if (error instanceof Error) {
		if (error.name === "ExitPromptError") {
			console.log("\nGracefully shutting down from SIGINT (Ctrl-C)");
		} else {
			console.error("An error occurred:", error.message);
			if (error.stack) {
				console.error(error.stack);
			}
			process.exit(1);
		}
	} else {
		console.error("An unexpected error occurred:", error);
		process.exit(1);
	}
};
