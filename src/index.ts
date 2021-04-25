import fs from "fs-extra";
import path from "path";
import execa from "execa";
import toml from "@iarna/toml";
import {
	glob,
	createLambda,
	debug,
	download,
	FileFsRef,
	runShellScript,
	BuildOptions,
	Meta,
	PrepareCacheOptions,
	DownloadedFiles
} from "@vercel/build-utils"; // eslint-disable-line import/no-extraneous-dependencies
import { installRustAndFriends } from "./install-rust";

interface CargoConfig {
	env: Record<string, any>;
	cwd: string;
}

const codegenFlags = [
	"-C",
	"target-cpu=ivybridge",
	"-C",
	"target-feature=-aes,-avx,+fxsr,-popcnt,+sse,+sse2,-sse3,-sse4.1,-sse4.2,-ssse3,-xsave,-xsaveopt"
];

export const version = 3;
const builderDebug = process.env.VERCEL_BUILDER_DEBUG ? true : false;

async function parseTOMLStream(stream: NodeJS.ReadableStream) {
	return toml.parse.stream(stream);
}

async function gatherExtraFiles(
	globMatcher: string | string[] | undefined,
	entrypoint: string
) {
	if (!globMatcher) return {};

	debug("Gathering extra files for the fs...");

	const entryDir = path.dirname(entrypoint);

	if (Array.isArray(globMatcher)) {
		const allMatches = await Promise.all(
			globMatcher.map(pattern => glob(pattern, entryDir))
		);

		return allMatches.reduce((acc, matches) => ({ ...acc, ...matches }), {});
	}

	return glob(globMatcher, entryDir);
}

async function runUserScripts(entrypoint: string) {
	const entryDir = path.dirname(entrypoint);
	const buildScriptPath = path.join(entryDir, "build.sh");
	const buildScriptExists = await fs.pathExists(buildScriptPath);

	if (buildScriptExists) {
		debug("Running `build.sh`...");
		await runShellScript(buildScriptPath);
	}
}

async function cargoLocateProject(config: CargoConfig) {
	try {
		const { stdout: projectDescriptionStr } = await execa(
			"cargo",
			["locate-project"],
			config
		);
		const projectDescription = JSON.parse(projectDescriptionStr);
		if (projectDescription != null && projectDescription.root != null) {
			return projectDescription.root;
		}
	} catch (e) {
		if (!/could not find/g.test(e.stderr)) {
			console.error("Couldn't run `cargo locate-project`");
			throw e;
		}
	}

	return null;
}

async function resolveBinary(
	meta: Meta,
	workPath: string,
	entrypointPath: string,
	cargoTomlFile: string,
	cargoToml: toml.JsonMap,
	buildBinary: (binName: string) => Promise<void>
) {
	const entrypointPathRelative = path.relative(workPath, entrypointPath)
	const bin = cargoToml.bin instanceof Array
		? (cargoToml.bin as toml.JsonMap[]).find((bin: toml.JsonMap) => bin.path === entrypointPathRelative)
		: null;
	let binName = bin && bin.name as string;

	if (bin == null) {
		binName = path
			.basename(entrypointPath)
			.replace(path.extname(entrypointPath), "")
			.replace("[", "_")
			.replace("]", "_");

		const tomlToWrite = toml.stringify({
			...cargoToml,
			bin: [{
				name: binName,
				path: entrypointPath
			}]
		});

		if (meta.isDev) {
			debug("Backing up Cargo.toml file");
			await fs.move(
				cargoTomlFile,
				`${cargoTomlFile}.backup`,
				{ overwrite: true }
			);
		}

		debug("Writing following toml to file:", tomlToWrite);
		try {
			await fs.writeFile(cargoTomlFile, tomlToWrite);
		} catch (error) {
			if (meta.isDev) {
				await restoreCargoToml(cargoTomlFile);
			}
			throw error;
		}
	}

	try {
		await buildBinary(binName as string);
	} catch (error) {
		if (bin == null && meta.isDev) {
			await restoreCargoToml(cargoTomlFile);
		}
	}

	if (bin == null && meta.isDev) {
		await restoreCargoToml(cargoTomlFile);
	}

	return binName as string;
}

async function restoreCargoToml(cargoTomlFile: any) {
	debug("Restoring backed up Cargo.toml file");
	await fs.move(`${cargoTomlFile}.backup`, cargoTomlFile, { overwrite: true });
}

async function buildSingleFile(
	{ entrypoint, workPath, meta = {} }: BuildOptions,
	downloadedFiles: DownloadedFiles,
	extraFiles: DownloadedFiles,
	rustEnv: Record<string, string>
) {
	debug("Building single file");
	const entrypointPath = downloadedFiles[entrypoint].fsPath;
	const entrypointDirname = path.dirname(entrypointPath);

	// Find a Cargo.toml file or TODO: create one
	const cargoTomlFile = await cargoLocateProject({
		env: rustEnv,
		cwd: entrypointDirname
	});

	// TODO: we're assuming there's a Cargo.toml file. We need to create one
	// otherwise
	let cargoToml: toml.JsonMap;
	try {
		cargoToml = (await parseTOMLStream(
			fs.createReadStream(cargoTomlFile)
		)) as toml.JsonMap;
	} catch (err) {
		console.error("Failed to parse TOML from entrypoint:", entrypoint);
		throw err;
	}

	const binName = await resolveBinary(meta, workPath, entrypointPath, cargoTomlFile, cargoToml, async binName => {
		debug("Running `cargo build`...");
		try {
			await execa(
				"cargo",
				["build", "--bin", binName].concat(
					builderDebug ? ["--verbose"] : ["--quiet", "--release"]
				),
				{
					env: rustEnv,
					cwd: entrypointDirname,
					stdio: "inherit"
				}
			);
		} catch (err) {
			console.error("failed to `cargo build`");
			throw err;
		}
	});

	// The compiled binary in Windows has the `.exe` extension
	const binExtension = process.platform === "win32" ? ".exe" : "";

	const bin = path.join(
		path.dirname(cargoTomlFile),
		"target",
		builderDebug ? "debug" : "release",
		binName + binExtension
	);

	debug("Binary file is: " + bin);

	const bootstrap = "bootstrap" + binExtension;

	const lambda = await createLambda({
		files: {
			...extraFiles,
			[bootstrap]: new FileFsRef({ mode: 0o755, fsPath: bin })
		},
		handler: bootstrap,
		runtime: "provided"
	});

	return { output: lambda };
}

export async function build(opts: BuildOptions) {
	await installRustAndFriends();

	const { files, entrypoint, workPath, config, meta = {} } = opts;
	debug("Downloading files");
	const downloadedFiles = await download(files, workPath, meta);
	const entryPath = downloadedFiles[entrypoint].fsPath;

	const { PATH, HOME } = process.env;
	const rustEnv: Record<string, string> = {
		...process.env,
		PATH: `${path.join(HOME!, ".cargo/bin")}:${PATH}`,
		RUSTFLAGS: [process.env.RUSTFLAGS, ...codegenFlags]
			.filter(Boolean)
			.join(" ")
	};

	await runUserScripts(entryPath);
	const extraFiles = await gatherExtraFiles(config.includeFiles, entryPath);

	return buildSingleFile(opts, downloadedFiles, extraFiles, rustEnv);
}

export async function prepareCache({
	cachePath,
	entrypoint,
	workPath
}: PrepareCacheOptions) {
	debug("Preparing cache...");

	let targetFolderDir: string;

	if (path.extname(entrypoint) === ".toml") {
		targetFolderDir = path.dirname(path.join(workPath, entrypoint));
	} else {
		const { PATH, HOME } = process.env;
		const rustEnv: Record<string, string> = {
			...process.env,
			PATH: `${path.join(HOME!, ".cargo/bin")}:${PATH}`,
			RUSTFLAGS: [process.env.RUSTFLAGS, ...codegenFlags]
				.filter(Boolean)
				.join(" ")
		};
		const entrypointDirname = path.dirname(path.join(workPath, entrypoint));
		const cargoTomlFile = await cargoLocateProject({
			env: rustEnv,
			cwd: entrypointDirname
		});

		if (cargoTomlFile != null) {
			targetFolderDir = path.dirname(cargoTomlFile);
		} else {
			// `Cargo.toml` doesn't exist, in `build` we put it in the same
			// path as the entrypoint.
			targetFolderDir = path.dirname(path.join(workPath, entrypoint));
		}
	}

	const cacheEntrypointDirname = path.join(
		cachePath,
		path.relative(workPath, targetFolderDir)
	);

	// Remove the target folder to avoid 'directory already exists' errors
	fs.removeSync(path.join(cacheEntrypointDirname, "target"));
	fs.mkdirpSync(cacheEntrypointDirname);
	// Move the target folder to the cache location
	fs.renameSync(
		path.join(targetFolderDir, "target"),
		path.join(cacheEntrypointDirname, "target")
	);

	const cacheFiles = await glob("**/**", cachePath);

	// eslint-disable-next-line no-restricted-syntax
	for (const f of Object.keys(cacheFiles)) {
		const accept =
			/(?:^|\/)target\/release\/\.fingerprint\//.test(f) ||
			/(?:^|\/)target\/release\/build\//.test(f) ||
			/(?:^|\/)target\/release\/deps\//.test(f) ||
			/(?:^|\/)target\/debug\/\.fingerprint\//.test(f) ||
			/(?:^|\/)target\/debug\/build\//.test(f) ||
			/(?:^|\/)target\/debug\/deps\//.test(f);
		if (!accept) {
			delete cacheFiles[f];
		}
	}

	return cacheFiles;
}

export { shouldServe } from "@vercel/build-utils";
