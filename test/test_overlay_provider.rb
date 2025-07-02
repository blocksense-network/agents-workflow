# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require_relative 'test_helper'
require_relative '../lib/snapshot/provider'

# Tests for OverlayFsProvider functionality
class TestOverlayProvider < Minitest::Test
  include RepoTestHelper

  def test_overlay_provider_detection
    skip 'OverlayFS tests only run on Linux' unless linux?

    repo, remote = setup_repo(:git)
    Snapshot::ZfsProvider.stub(:available?, false) do
      Snapshot::BtrfsProvider.stub(:available?, false) do
        provider = Snapshot.provider_for(repo)
        # Provider should fall back to OverlayFs when available
        assert_kind_of Snapshot::OverlayFsProvider, provider
      end
    end
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_create_and_cleanup_workspace
    skip 'OverlayFS tests only run on Linux' unless linux?
    skip 'OverlayFS not available' unless Snapshot::OverlayFsProvider.available?('.')

    repo, remote = setup_repo(:git)
    provider = Snapshot::OverlayFsProvider.new(repo)
    dest = Dir.mktmpdir('overlay_ws')
    workspace_created = false
    begin
      ws_path = provider.create_workspace(dest)
      workspace_created = true
    rescue RuntimeError => e
      skip "Overlay mount failed: #{e.message}"
    end
    # Workspace should contain README from repo
    assert File.exist?(File.join(ws_path, 'README.md'))
    # Adding file in workspace should not appear in repo
    File.write(File.join(ws_path, 'new.txt'), 'overlay')
    refute File.exist?(File.join(repo, 'new.txt'))
  ensure
    provider.cleanup_workspace(dest) if workspace_created && dest && File.exist?(dest)
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end
end
