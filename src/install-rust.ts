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

async function installOpenSSL() {
	console.log("installing openssl-devel...");
	try {
		await execa("yum", ["install", "-y", "openssl-devel"], {
			stdio: "inherit"
		});
	} catch (err) {
		console.error("failed to `yum install -y openssl-devel`");
		throw err;
	}
}

export const installRustAndFriends = async (version?: string) => {
	await downloadRustToolchain(version);
	await installOpenSSL();
};

installRustAndFriends().catch(err => {
	console.error(err);
	process.exit(1);
});
