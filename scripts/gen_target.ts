const TARGET = Deno.args[0] || "x86_64-unknown-none";

const rustc = Deno.run({
    cmd: [
        "rustc",
        "+nightly",
        "-Z",
        "unstable-options",
        "--print",
        "target-spec-json",
        "--target",
        TARGET,
    ],
    stdout: "inherit",
    stderr: "inherit",
});

await rustc.status();
