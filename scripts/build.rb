# frozen_string_literal: true

ARG_REG = /^--(?<name>[^=]+)(?<option>=(?<value>[^=]+))*$/
ROOT_DIR = File.join(__dir__, '..')

target = 'x86_64-unknown-ingram.json'

command = %w[cargo build]

ARGV.each do |arg|
  arg.match(ARG_REG) do |m|
    case m[:name]
    when 'arch'
      target = "#{m[:value]}-unknown-ingram.json"
    when 'release'
      command << '--release'
    end
  end
end

command << '--target' << target

pid = Process.spawn(*command, chdir: ROOT_DIR)
Process.wait pid
