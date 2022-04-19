import {
  basename,
  dirname,
  join,
} from "https://deno.land/std@0.133.0/path/mod.ts";

export { basename, dirname, join };

export const TEST = Deno.args.includes("--test");
export const MODE: "debug" | "release" = Deno.args.includes("--release")
  ? "release"
  : "debug";
export const PROD = MODE === "release";

export const PKG = "ingram";
export const TARGET = "x86_64-unknown-none";

export const ROOT_DIR = Deno.cwd();
export const DIST_DIR = join(ROOT_DIR, "dist");
export const USER_DIR = join(ROOT_DIR, "user");
export const KERNEL_DIR = join(ROOT_DIR, "kernel");
export const TARGET_DIR = join(ROOT_DIR, "target");
export const TARGET_BIN_DIR = join(TARGET_DIR, TARGET, MODE);
