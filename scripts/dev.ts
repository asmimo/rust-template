import type { ReadonlyDeep } from "type-fest";

import { type AppConfig, getConfig } from "./config.ts";
import { getCargoTOML } from "./fs.ts";
import { catchError, spawnSafe } from "./process.ts";
import { getApp, getAppFeatures } from "./prompts.ts";

const run = async (config: ReadonlyDeep<AppConfig>): Promise<void> => {
	const app = await getApp(config.app);
	const cargoToml = await getCargoTOML(`app/${app}`);

	if (cargoToml) {
		const appFeatures = await getAppFeatures(cargoToml?.features, config.features);
		if (appFeatures.length > 0) {
			process.env.FEATURES = appFeatures.join(",");
		}
	}

	console.inspect(config);
	await spawnSafe("turbo", ["watch", "dev", `--filter=${app}`], {
		env: {
			APP: app,
			...process.env,
		},
	});
};

try {
	const config = await getConfig();

	await run(config);
} catch (error) {
	catchError(error);
}
