import {
  basename,
  dirname,
  join,
  KERNEL_DIR,
  PKG,
  PROD,
  TARGET_BIN_DIR,
  TARGET_DIR,
  TEST,
} from "./env.ts";
import { buildjs } from "./build_js.ts";

const textDecoder = new TextDecoder();

buildjs();

export const images = await (async () => {
  {
    const cmd = ["cargo"];
    if (TEST) cmd.push("test", "--no-run");
    else cmd.push("build");
    if (PROD) cmd.push("--release");

    const build = Deno.run({ cmd, stdout: "inherit", stderr: "inherit" });
    if (!(await build.status()).success) throw new Error("build failed");

    const mainBin = join(TARGET_BIN_DIR, PKG);
    const kernelBinaryPaths: string[] = [];
    if (!TEST) kernelBinaryPaths.push(mainBin);
    else {
      cmd.push("--message-format", "json");
      const res = Deno.run({ cmd, stdout: "piped", stderr: "inherit" });
      if (!(await res.status()).success) throw new Error("build failed");

      for (const line of textDecoder.decode(await res.output()).split("\n")) {
        if (!line) continue;

        const json = JSON.parse(line) as { executable?: null | string };
        if (typeof json.executable !== "string") continue;
        if (json.executable === mainBin) continue;
        if (json.executable.startsWith(`${TARGET_BIN_DIR}/deps/${PKG}-`)) {
          continue;
        }

        kernelBinaryPaths.push(json.executable);
      }

      // Build tests also build the `main.rs`. Ignore it.
      const mainSize = (await Deno.stat(mainBin)).size;
      for (let i = kernelBinaryPaths.length - 1; i >= 0; --i) {
        if ((await Deno.stat(kernelBinaryPaths[i])).size === mainSize) {
          kernelBinaryPaths.splice(i, 1);
          break;
        }
      }
    }

    return createDiskImages(kernelBinaryPaths);
  }
})();

/**
 * Copies from {@link https://github.com/rust-osdev/bootloader/blob/a1286cab072ad03a3d302be9ea694e3f0d72aa9e/examples/test_framework/boot/src/main.rs#L70}
 */
async function createDiskImages(kernelBinaryPaths: string[]) {
  console.log("Creating disk images...");

  const kernelManifest = join(KERNEL_DIR, "Cargo.toml");

  const images = [];
  for (const kernelBinary of kernelBinaryPaths) {
    const outDir = dirname(kernelBinary);

    const build = Deno.run({
      // deno-fmt-ignore
      cmd: [
        "cargo", "builder",
        "--kernel-manifest", kernelManifest,
        "--kernel-binary", kernelBinary,
        "--firmware", "uefi",
        "--target-dir", TARGET_DIR,
        "--out-dir", outDir,
      ],
      cwd: dirname(await locateBootloader()),
      stdout: "inherit",
      stderr: "inherit",
    });

    const status = await build.status();
    if (!status.success) throw new Error("build failed");

    const image = join(outDir, `boot-uefi-${basename(kernelBinary)}.img`);
    await Deno.stat(image);
    console.log(`Created disk image: ${image}`);
    images.push(image);
  }

  return images;
}

/**
 * Copies from {@link https://docs.rs/crate/bootloader-locator/0.0.4/source/src/lib.rs}
 */
async function locateBootloader() {
  const cmd = Deno.run({
    cmd: ["cargo", "metadata", "--format-version", "1"],
    stdout: "piped",
    stderr: "inherit",
    cwd: KERNEL_DIR,
  });

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
