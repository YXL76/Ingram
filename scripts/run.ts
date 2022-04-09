import {images, join, ROOT_DIR, TEST} from "./build.ts";

/** Follow {@link https://gil0mendes.io/blog/an-efi-app-a-bit-rusty/} */
// deno-fmt-ignore
const cmd = [
  "qemu-system-x86_64",
  "-enable-kvm",
  "--no-reboot",
  // Disable default devices
  "-nodefaults",
  // Use a standard VGA for graphics
  "-vga", "std",
  // Use a modern machine, with acceleration if possible.
  "-machine", "q35,accel=kvm:tcg",
  // Allocate some memory
  "-m", "2G",
  // Set up OVMF
  "-bios", join(ROOT_DIR, "OVMF.fd"),
  // Enable serial
  "-serial", "stdio",
  // Setup monitor
  "-monitor", "vc:1024x768",
  // Disable display
  "-display", "none",
  //  prevent attempting a PXE (network) boot when no boot disk is found
  "-net", "none",
  // Passthrough host CPU
  "-cpu", "host",
  // Only support single core
  "-smp", "1,maxcpus=1"
];

if (TEST) {
  cmd.push("-device", "isa-debug-exit,iobase=0xf4,iosize=0x04");

  for (const image of images) {
    const run = Deno.run({
      cmd: [
        ...cmd,
        "-drive",
        `format=raw,file=${image}`,
      ],
      stdout: "inherit",
      stderr: "inherit",
    });
    const status = await run.status();

    if (status.code !== 0x21) throw new Error("Test failed");
  }
} else {
  const run = Deno.run({
    cmd: [
      ...cmd,
      "-drive",
      `format=raw,file=${images[0]}`,
    ],
    stdout: "inherit",
    stderr: "inherit",
  });

  const status = await run.status();
  if (!status.success) Deno.exit(status.code);
}
