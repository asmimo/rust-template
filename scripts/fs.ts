import fs from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";

export const __filename = fileURLToPath(import.meta.url);
export const __dirname = dirname(__filename);

export const getDirectoryFolders = async (
	dir: string,
	ignoreDotFiles: boolean = true,
) => {
	const resolved = join(__dirname, dir);

	return fs.readdir(resolved).then((f) => {
		if (ignoreDotFiles) {
			return f.filter((f) => !f.startsWith(".")).sort();
		}
		return f;
	});
};

export const getCargoTOML = async (path: string) => {
	const tomlPath = join(__dirname, "../", path, "Cargo.toml");

	try {
		await fs.access(tomlPath);
	} catch {
		return undefined;
	}

	const tomlContent = await fs.readFile(tomlPath, "utf-8");
	const toml = parse(tomlContent);

	return toml;
};

export const getDockerfile = async (app: string) => {
	const dockerfilePath = join(__dirname, "../app", app, "Dockerfile");

	try {
		await fs.access(dockerfilePath);
	} catch {
		return undefined;
	}

	return fs.readFile(dockerfilePath, "utf-8");
};

export const getPackageJSON = async (
	app: string,
): Promise<unknown | undefined> => {
	const packageJsonPath = join(__dirname, "../app", app, "package.json");

	try {
		await fs.access(packageJsonPath);
	} catch {
		return undefined;
	}

	const content = await fs.readFile(packageJsonPath, "utf-8");
	return JSON.parse(content);
};
