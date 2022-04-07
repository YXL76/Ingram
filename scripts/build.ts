import {
  basename,
  dirname,
  join,
} from "https://deno.land/std@0.133.0/path/mod.ts";

export { join };

const textDecoder = new TextDecoder();

export const TEST = Deno.args.includes("--test");
const MODE: "debug" | "release" = Deno.args.includes("--release")
  ? "release"
  : "debug";

const PKG = "ingram";
const TARGET = "x86_64-unknown-none";

export const ROOT_DIR = Deno.cwd();
const TARGET_DIR = join(ROOT_DIR, "target");

export const imagePath = await (async () => {
  {
    const cmd = ["cargo", "build"];
    if (MODE === "release") cmd.push("--release");

    let kernelBinaryPath: string;
    if (TEST) {
      cmd.push("--tests");

      const plan = Deno.run({
        cmd: [...cmd, "-Z", "unstable-options", "--build-plan"],
        stdout: "piped",
        stderr: "inherit",
      });

      const { invocations } = JSON.parse(
        textDecoder.decode(await plan.output()),
      ) as { invocations: { package_name: string; outputs: string[] }[] };

      const pkg = invocations.find((i) => i.package_name === PKG);
      if (!pkg) throw new Error("Not Found");

      kernelBinaryPath = pkg.outputs[0];
    } else {
      kernelBinaryPath = join(TARGET_DIR, TARGET, MODE, PKG);
    }

    const build = Deno.run({
      cmd,
      stdout: "inherit",
      stderr: "inherit",
    });
    if (!(await build.status()).success) {
      throw new Error("build failed");
    }

    return createDiskImages(kernelBinaryPath);
  }
})();

/**
 * Copies from {@link https://github.com/rust-osdev/bootloader/blob/a1286cab072ad03a3d302be9ea694e3f0d72aa9e/examples/test_framework/boot/src/main.rs#L70}
 */
async function createDiskImages(kernelBinaryPath: string) {
  console.log("Creating disk images...");

  const kernelBinaryDir = dirname(kernelBinaryPath);
  const kernelManifestPath = join(ROOT_DIR, "Cargo.toml");

  const cmd = [
    "cargo",
    "builder",

    "--kernel-manifest",
    kernelManifestPath,

    "--kernel-binary",
    kernelBinaryPath,

    "--firmware",
    "uefi",

    "--target-dir",
    TARGET_DIR,

    "--out-dir",
    kernelBinaryDir,
  ];
  const build = Deno.run({
    cmd,
    cwd: dirname(await locateBootloader()),
    stdout: "inherit",
    stderr: "inherit",
  });

  const status = await build.status();
  if (!status.success) throw new Error("build failed");

  const image = join(
    kernelBinaryDir,
    `boot-uefi-${basename(kernelBinaryPath)}.img`,
  );
  await Deno.stat(image);
  console.log(`Created disk image: ${image}`);
  return image;
}

/**
 * Copies from {@link https://docs.rs/crate/bootloader-locator/0.0.4/source/src/lib.rs}
 */
async function locateBootloader() {
  const cmd = Deno.run({
    cmd: ["cargo", "metadata", "--format-version", "1"],
    stdout: "piped",
    stderr: "piped",
  });
  const status = await cmd.status();
  if (!status.success) {
    throw new Error(textDecoder.decode(await cmd.stderrOutput()));
  }

  const metadata = JSON.parse(textDecoder.decode(await cmd.output())) as {
    packages: { id: string; manifest_path: string }[];
    resolve: {
      nodes: { id: string; deps: { name: string; pkg: string }[] }[];
      root: string;
    };
  };

  const { root } = metadata.resolve;

  const rootRsv = metadata.resolve.nodes.find(({ id }) => id === root);
  if (!rootRsv) throw new Error("Not Found");

  const dependency = rootRsv.deps.find(({ name }) => name === "bootloader");
  if (!dependency) throw new Error("Not Found");

  const depId = dependency.pkg;

  const dependencyPackage = metadata.packages.find(({ id }) => id === depId);
  if (!dependencyPackage) throw new Error("Not Found");

  return dependencyPackage.manifest_path;
}
