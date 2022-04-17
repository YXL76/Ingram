import { build, stop } from "https://deno.land/x/esbuild@v0.14.36/mod.js";
import type { BuildOptions } from "https://deno.land/x/esbuild@v0.14.36/mod.js";

const basicConfig: BuildOptions = {
  sourcemap: false,
  legalComments: "none",
  color: true,
  format: "esm",
  logLevel: "warning",
  target: "es2016",
  minify: false,

  bundle: true,
  splitting: false,
  outdir: "/home/yxl/Ingram/dist",
  platform: "neutral",
  loader: { ".ts": "ts", ".js": "js", ".cjs": "js", ".mjs": "js" },
  outExtension: { ".js": ".js" },
  banner: { js: "'use strict';" },
  entryPoints: ["kernel/js/index.ts"],
};

export async function buildjs(prod: boolean) {
  await build({
    ...basicConfig,

    minify: prod,
  });
  stop();
}
