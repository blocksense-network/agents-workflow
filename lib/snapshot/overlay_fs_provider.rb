# frozen_string_literal: true

module Snapshot
  # OverlayFS snapshot using a temporary overlay mount
  class OverlayFsProvider < Provider
    def self.available?(_path)
      File.read('/proc/filesystems').include?('overlay')
    end

    def create_workspace(dest)
      upper = File.join(dest, 'upper')
      work = File.join(dest, 'work')
      merged = File.join(dest, 'merged')
      FileUtils.mkdir_p([upper, work, merged])
      options = "lowerdir=#{@repo_path},upperdir=#{upper},workdir=#{work}"
      run('mount', '-t', 'overlay', 'overlay', '-o', options, merged)
      merged
    end

    def cleanup_workspace(dest)
      merged = File.join(dest, 'merged')
      run('umount', merged)
      FileUtils.rm_rf(dest)
    end

    private

    def run(*cmd)
      system(*cmd) || raise("Command failed: #{cmd.join(' ')}")
    end
  end
end
