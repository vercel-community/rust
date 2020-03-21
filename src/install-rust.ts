import execa from "execa";

async function downloadRustToolchain(version: string = "stable") {
	console.log("downloading the rust toolchain");

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
	await downloadRustToolchain(version);
};
