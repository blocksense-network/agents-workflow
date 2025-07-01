# frozen_string_literal: true

module Snapshot
  # Simple copy-based provider
  class CopyProvider < Provider
    def self.available?(_path)
      true
    end

    def create_workspace(dest)
      FileUtils.mkdir_p(dest)
      run('cp', '-a', '--reflink=auto', File.join(@repo_path, '.'), dest)
      dest
    end

    def cleanup_workspace(dest)
      FileUtils.rm_rf(dest)
    end

    private

    def run(*cmd)
      system(*cmd) || raise("Command failed: #{cmd.join(' ')}")
    end
  end
end
