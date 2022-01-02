# frozen_string_literal: true

require 'fileutils'

ARG_REG = /^--(?<name>[^=]+)(?<option>=(?<value>[^=]+))*$/
ROOT_DIR = File.join(__dir__, '..')
BUILD_DIR = File.join(ROOT_DIR, 'build')
OVMF = File.join(BUILD_DIR, 'OVMF.fd')
BOOT_DIR = File.join(BUILD_DIR, 'EFI', 'BOOT')
IMAGE = File.join(BOOT_DIR, 'BOOT.EFI')

mod = 'debug'
ovmf = '/usr/share/edk2-ovmf/x64/OVMF.fd'
qemu = 'qemu-system-x86_64'
target = 'x86_64-unknown-uefi'

ARGV.each do |arg|
  arg.match(ARG_REG) do |m|
    case m[:name]
    when 'arch'
      target = "#{m[:value]}-unknown-uefi"
      # ovmf =
      qemu = "qemu-system-#{m[:value]}"
    when 'release'
      mod = 'release'
    end
  end
end

File.copy_stream(ovmf, OVMF)
FileUtils.mkdir_p(BOOT_DIR) unless File.exist?(BOOT_DIR)
File.copy_stream(File.join(ROOT_DIR, 'target', target, mod, 'ingram.efi'), IMAGE)

# Follow https://gil0mendes.io/blog/an-efi-app-a-bit-rusty/
command = [qemu,
           '-enable-kvm',

           # Disable default devices
           '-nodefaults',

           # Use a standard VGA for graphics
           '-vga', 'std',

           # Use a modern machine, with acceleration if possible.
           '-machine', 'q35,accel=kvm:tcg',

           # Allocate some memory
           '-m', '4G',

           # Set up OVMF
           '-bios', OVMF,
           # '-drive', "if=pflash,format=raw,unit=0,file=#{ovmf_code},readonly=on",
           # '-drive', "if=pflash,format=raw,unit=1,file=#{ovmf_vars}",

           # Mount a local directory as a FAT partition
           '-drive', "format=raw,file=fat:rw:#{BUILD_DIR}",

           # Enable serial
           '-serial', 'stdio',

           # Setup monitor
           '-monitor', 'vc:1024x768',

           #  prevent attempting a PXE (network) boot when no boot disk is found
           '-net', 'none']

pid = Process.spawn(*command, chdir: ROOT_DIR)
Process.wait pid
