import fs from "node:fs/promises";
import path from "node:path";

import { type TomlTable, parse } from "smol-toml";
import type { PackageJson } from "type-fest";

export const getDirectoryFolders = async (
	dir: string,
	ignoreDotFiles = true,
): Promise<string[]> => {
	const resolved = path.join(import.meta.dirname, dir);

	return await fs.readdir(resolved).then((files) => {
		const sortedFiles = files.toSorted();
		if (ignoreDotFiles) {
			return sortedFiles.filter((file) => !file.startsWith("."));
		}
		return sortedFiles;
	});
};

export const getTOML = async (dir: string): Promise<TomlTable | undefined> => {
	const tomlPath = path.join(import.meta.dirname, dir);

	try {
		await fs.access(tomlPath);
	} catch {
		return;
	}

	const tomlContent = await fs.readFile(tomlPath, "utf8");

	return parse(tomlContent);
};

export const getCargoTOML = async (dir: string): Promise<TomlTable | undefined> =>
	await getTOML(`../${dir}/Cargo.toml`);

export const getDockerfile = async (app: string): Promise<string | undefined> => {
	const dockerfilePath = path.join(import.meta.dirname, "../app", app, "Dockerfile");

	try {
		await fs.access(dockerfilePath);
	} catch {
		return;
	}

	return fs.readFile(dockerfilePath, "utf8");
};

export const getPackageJSON = async (app: string): Promise<PackageJson | undefined> => {
	const packageJsonPath = path.join(import.meta.dirname, "../app", app, "package.json");

	try {
		await fs.access(packageJsonPath);
	} catch {
		return;
	}

	const content = await fs.readFile(packageJsonPath, "utf8");
	return JSON.parse(content);
};
