# frozen_string_literal: true

require 'shellwords'

module Snapshot
  # Btrfs subvolume snapshot implementation
  class BtrfsProvider < Provider
    def self.available?(path)
      # Btrfs is only available on Linux in this implementation
      return false unless RUBY_PLATFORM.include?('linux')

      system('which', 'btrfs', out: File::NULL, err: File::NULL) &&
        fs_type(path) == 'btrfs'
    end

    def create_workspace(dest)
      run('btrfs', 'subvolume', 'snapshot', @repo_path, dest)
      dest
    end

    def cleanup_workspace(dest)
      run('btrfs', 'subvolume', 'delete', dest)
    end

    def self.fs_type(path)
      `stat -f -c %T #{Shellwords.escape(path)}`.strip
    end
    private_class_method :fs_type

    private

    def run(*cmd)
      system(*cmd) || raise("Command failed: #{cmd.join(' ')}")
    end
  end
end
