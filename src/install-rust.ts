import execa from "execa";
import { debug } from "@now/build-utils";

async function downloadRustToolchain(version: string = "stable") {
	debug("Downloading the rust toolchain");

	try {
		await execa.shell(
			`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${version}`,
			{ stdio: "inherit" }
		);
	} catch (err) {
		throw new Error(`Failed to install rust via rustup: ${err.message}`);
	}
}

export const installRustAndFriends = async (version?: string) => {
	try {
		await execa.shell(`rustup -V`, { stdio: "ignore" });
		debug("Rust already exists");
	} catch (err) {
		await downloadRustToolchain(version);
	}
};
