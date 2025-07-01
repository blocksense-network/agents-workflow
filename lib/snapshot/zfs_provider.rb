# frozen_string_literal: true

require 'shellwords'

module Snapshot
  # ZFS snapshot implementation
  class ZfsProvider < Provider
    def self.available?(path)
      system('which', 'zfs', out: File::NULL, err: File::NULL) &&
        fs_type(path) == 'zfs'
    end

    def create_workspace(dest)
      dataset = dataset_for(@repo_path)
      raise 'ZFS dataset not found' unless dataset

      tag = "agent#{Time.now.to_i}"
      snapshot = "#{dataset}@#{tag}"
      clone = "#{dataset}-clone-#{tag}"
      run('zfs', 'snapshot', snapshot)
      run('zfs', 'clone', snapshot, clone)
      run('zfs', 'set', "mountpoint=#{dest}", clone)
      dest
    end

    def cleanup_workspace(dest)
      dataset = dataset_for(dest)
      run('zfs', 'destroy', '-r', dataset) if dataset
    end

    def self.fs_type(path)
      `stat -f -c %T #{Shellwords.escape(path)}`.strip
    end
    private_class_method :fs_type

    private

    def dataset_for(path)
      list = `zfs list -H -o name,mountpoint`
      best = list.lines.map(&:split)
                 .select { |_, mount| path.start_with?(mount) }
                 .max_by { |_, mount| mount.length }
      best&.first
    end

    def run(*cmd)
      system(*cmd) || raise("Command failed: #{cmd.join(' ')}")
    end
  end
end
