# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require_relative 'test_helper'
require_relative '../lib/snapshot/provider'

# Tests for snapshot provider detection and basic copy provider
class TestSnapshotProvider < Minitest::Test
  include RepoTestHelper

  def test_detection_returns_provider
    repo, remote = setup_repo(:git)
    provider = Snapshot.provider_for(repo)
    # provider should be some Snapshot::Provider subclass
    assert_kind_of Snapshot::Provider, provider

    # On non-Linux systems, should fall back to CopyProvider
    assert_kind_of Snapshot::CopyProvider, provider if macos? || windows?
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_copy_provider_creates_workspace
    repo, remote = setup_repo(:git)
    provider = Snapshot::CopyProvider.new(repo)
    dest = Dir.mktmpdir('ws')
    provider.create_workspace(dest)
    # File from repo should exist in the workspace
    assert File.exist?(File.join(dest, 'README.md'))
  ensure
    provider.cleanup_workspace(dest) if dest && File.exist?(dest)
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end
end
