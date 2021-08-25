import execa from 'execa';
import { debug } from '@vercel/build-utils';

async function downloadRustToolchain(version: string = 'stable') {
  debug('Downloading the rust toolchain');

  try {
    await execa(
      `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${version}`,
      [],
      { shell: true, stdio: 'inherit' }
    );
  } catch (err) {
    throw new Error(`Failed to install rust via rustup: ${err.message}`);
  }
}

export const installRustAndFriends = async (version?: string) => {
  try {
    await execa(`rustup -V`, [], { shell: true, stdio: 'ignore' });
    debug('Rust already exists');
  } catch (err) {
    await downloadRustToolchain(version);
  }
};
