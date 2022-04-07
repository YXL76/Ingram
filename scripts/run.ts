import { imagePath, join, ROOT_DIR, TEST } from "./build.ts";

const cmd = [
  "qemu-system-x86_64",

  "-enable-kvm",

  "--no-reboot",

  // Disable default devices
  "-nodefaults",

  // Use a standard VGA for graphics
  "-vga",
  "std",

  // Use a modern machine, with acceleration if possible.
  "-machine",
  "q35,accel=kvm:tcg",

  // Allocate some memory
  "-m",
  "4G",

  // Set up OVMF
  "-bios",
  join(ROOT_DIR, "OVMF.fd"),

  // Enable serial
  "-serial",
  "stdio",

  // Setup monitor
  "-monitor",
  "vc:1024x768",

  "-display",
  "none",

  //  prevent attempting a PXE (network) boot when no boot disk is found
  "-net",
  "none",

  // Mount image
  "-drive",
  `format=raw,file=${imagePath}`,
];

if (TEST) {
  cmd.push("-device", "isa-debug-exit,iobase=0xf4,iosize=0x04");
}

const run = Deno.run({
  /** Follow {@link https://gil0mendes.io/blog/an-efi-app-a-bit-rusty/} */
  cmd,
  stdout: "inherit",
  stderr: "inherit",
});

const status = await run.status();
if (TEST) {
  if (status.code !== 0x21) throw new Error("Test failed");
} else {
  if (!status.success) Deno.exit(status.code);
}
