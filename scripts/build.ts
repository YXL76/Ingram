import { join, dirname } from "https://deno.land/std@0.133.0/path/mod.ts";

export { join };

const textDecoder = new TextDecoder();

const mode: "debug" | "release" = Deno.args.includes("--release")
    ? "release"
    : "debug";

const BINARY = "ingram";
const TARGET = "x86_64-unknown-none";

export const ROOT_DIR = Deno.cwd();
const TARGET_DIR = join(ROOT_DIR, "target");

{
    const cmd = ["cargo", "build"];
    if (mode === "release") cmd.push("--release");
    const build = Deno.run({
        cmd,
        stdout: "inherit",
        stderr: "inherit",
    });
    if (!(await build.status()).success) {
        throw new Error("build failed");
    }
}

export const imagePath = await createDiskImages();

/**
 * Copies from {@link https://github.com/rust-osdev/bootloader/blob/a1286cab072ad03a3d302be9ea694e3f0d72aa9e/examples/test_framework/boot/src/main.rs#L70}
 */
async function createDiskImages() {
    console.log("Creating disk images...");

    const kernelBinaryDir = join(TARGET_DIR, TARGET, mode);
    const kernelBinaryPath = join(kernelBinaryDir, BINARY);
    const kernelManifestPath = join(ROOT_DIR, "Cargo.toml");

    const cmd = [
        "cargo",
        "builder",
        "--kernel-manifest",
        kernelManifestPath,
        "--kernel-binary",
        kernelBinaryPath,
        "--target-dir",
        TARGET_DIR,
        "--out-dir",
        kernelBinaryDir,
        "--quiet",
    ];
    const build = Deno.run({
        cmd,
        cwd: dirname(await locateBootloader()),
        stdout: "inherit",
        stderr: "inherit",
    });

    const status = await build.status();
    if (!status.success) {
        throw new Error("build failed");
    }

    const image = join(kernelBinaryDir, `boot-uefi-${BINARY}.img`);
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
