# frozen_string_literal: true

require 'fileutils'

module Snapshot
  # Detect and return appropriate provider for given repository path
  def self.provider_for(path)
    path = File.expand_path(path)
    return ZfsProvider.new(path) if ZfsProvider.available?(path)
    return BtrfsProvider.new(path) if BtrfsProvider.available?(path)
    return OverlayFsProvider.new(path) if OverlayFsProvider.available?(path)

    CopyProvider.new(path)
  end

  # Base class for snapshot providers
  class Provider
    attr_reader :repo_path

    def initialize(repo_path)
      @repo_path = File.expand_path(repo_path)
    end

    # Create an isolated workspace rooted at dest
    def create_workspace(dest)
      raise NotImplementedError
    end

    # Cleanup any resources associated with the workspace
    def cleanup_workspace(_dest)
      # optional
    end
  end

  require_relative 'zfs_provider'
  require_relative 'btrfs_provider'
  require_relative 'overlay_fs_provider'
  require_relative 'copy_provider'
end
