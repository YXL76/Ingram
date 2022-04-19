import { DIST_DIR, join, KERNEL_DIR, PROD, USER_DIR } from "./env.ts";
import { build, stop } from "https://deno.land/x/esbuild@v0.14.36/mod.js";
import type { BuildOptions } from "https://deno.land/x/esbuild@v0.14.36/mod.js";

const basicConfig: BuildOptions = {
  sourcemap: false,
  legalComments: "none",
  color: true,
  format: "esm",
  logLevel: "warning",
  target: "es2021",
  minify: false,

  bundle: true,
  splitting: false,
  outdir: DIST_DIR,
  platform: "neutral",
  loader: { ".ts": "ts", ".js": "js", ".cjs": "js", ".mjs": "js" },
  outExtension: { ".js": ".js" },
  banner: { js: "'use strict';" },
  entryPoints: [join(KERNEL_DIR, "js", "index.ts")],
};

export async function buildjs() {
  const entryPoints: Record<string, string> = {};
  for await (const { name } of Deno.readDir(USER_DIR)) {
    entryPoints[name] = join(USER_DIR, name);
  }

  const { outputFiles } = await build({
    ...basicConfig,

    write: false,
    minify: PROD,
    entryPoints,
  });

  await build({
    ...basicConfig,

    define: {
      USER_CODES: JSON.stringify(
        outputFiles.map(({ contents }) => [...contents]),
      ),
    },
    minify: PROD,
  });
  stop();
}
