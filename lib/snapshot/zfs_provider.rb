# frozen_string_literal: true

require 'shellwords'

module Snapshot
  # ZFS snapshot implementation
  class ZfsProvider < Provider
    def self.available?(path)
      # ZFS is only available on Linux in this implementation
      return false unless RUBY_PLATFORM.include?('linux')

      system('which', 'zfs', out: File::NULL, err: File::NULL) &&
        fs_type(path) == 'zfs'
    end

    def create_workspace(dest)
      # Validate destination path
      validate_destination_path(dest)

      dataset = dataset_for(@repo_path)
      raise 'ZFS dataset not found' unless dataset

      tag = "agent#{Process.pid}_#{Time.now.to_i}"
      snapshot = "#{dataset}@#{tag}"
      clone = "#{dataset}-clone-#{tag}"
      run('zfs', 'snapshot', snapshot)
      # ZFS clone may succeed but return exit code 1 if mounting requires root
      clone_success = system('zfs', 'clone', snapshot, clone)
      unless clone_success
        # Check if the clone was actually created despite the error
        clone_exists = system('zfs', 'list', clone, out: File::NULL, err: File::NULL)
        raise("Command failed: zfs clone #{snapshot} #{clone}") unless clone_exists
      end

      # ZFS clone is created, now check if it's mounted and accessible
      actual_mountpoint = `zfs get -H -o value mountpoint #{clone} 2>/dev/null`.strip

      # Check if the clone is mounted and accessible
      if actual_mountpoint != 'none' && actual_mountpoint != 'legacy' && Dir.exist?(actual_mountpoint)
        repo_has_readme = File.exist?(File.join(@repo_path, 'README.md'))
        clone_has_readme = File.exist?(File.join(actual_mountpoint, 'README.md'))

        if repo_has_readme == clone_has_readme
          # Clone appears to be properly mounted and accessible
          # Create a symlink from dest to the actual mountpoint
          FileUtils.mkdir_p(File.dirname(dest))
          File.symlink(actual_mountpoint, dest) unless File.exist?(dest)
          return dest
        end
      end

      # Clone is not properly accessible - this indicates ZFS mounting permissions issue
      raise 'ZFS clone created but not accessible - check ZFS mounting permissions'
    end

    def cleanup_workspace(dest)
      # Find and destroy ZFS datasets
      actual_path = begin
        File.realpath(dest)
      rescue StandardError
        dest
      end

      dataset = dataset_for(actual_path)
      dataset ||= dataset_for(dest)
      begin
        run('zfs', 'destroy', '-r', dataset)
      rescue StandardError
        nil
      end

      # Remove the symlink if it exists
      File.unlink(dest) if File.symlink?(dest)
    end

    def self.fs_type(path)
      `stat -f -c %T #{Shellwords.escape(path)}`.strip
    end
    private_class_method :fs_type

    private

    def validate_destination_path(dest)
      # Check if the destination path can be created as a directory
      begin
        # Try to create the parent directory to validate the path
        parent_dir = File.dirname(dest)
        FileUtils.mkdir_p(parent_dir)
        # Clean up the test directory
        Dir.rmdir(parent_dir) if Dir.exist?(parent_dir) && Dir.empty?(parent_dir)
      rescue StandardError => e
        raise "Invalid destination path: #{dest} (#{e.message})"
      end

      # Additional validation: ensure it's not trying to create in system directories
      # that should not be used for workspaces
      invalid_paths = ['/dev', '/proc', '/sys', '/run']
      return unless invalid_paths.any? { |invalid| dest.start_with?(invalid) }

      raise "Cannot create workspace in system directory: #{dest}"
    end

    def dataset_for(path)
      list = `zfs list -H -o name,mountpoint 2>/dev/null`
      candidates = list.lines.map(&:split)
                       .select { |_name, mount| mount != 'none' && mount != 'legacy' && mount != '/' && path.start_with?(mount) }

      # Find the dataset with the longest mountpoint that actually contains the path
      best = candidates.max_by { |_, mount| mount.length }

      # Only return the dataset if the path actually exists within it
      return unless best && File.exist?(path)

      best.first
    end

    def run(*cmd)
      system(*cmd) || raise("Command failed: #{cmd.join(' ')}")
    end
  end
end
