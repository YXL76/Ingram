# frozen_string_literal: true

TARGET = ARGV.length.positive? ? ARGV[0] : 'x86_64-unknown-uefi'

pid = Process.spawn("rustc +nightly -Z unstable-options --print target-spec-json --target #{TARGET}")
Process.wait pid
