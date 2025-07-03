# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative '../test_helper'
require_relative 'provider_shared_behavior'
require_relative 'provider_quota_test_behavior'
require_relative 'provider_loop_device_test_behavior'
require_relative 'filesystem_test_helper'
require_relative '../../lib/snapshot/provider'

# Comprehensive tests for Copy provider combining generic and specific tests
class TestCopyProvider < Minitest::Test
  include RepoTestHelper
  include FilesystemTestHelper
  include ProviderSharedBehavior

  def setup
    @test_dir = Dir.mktmpdir('copy_test')
    @filesystems = []
    @mount_points = []

    # For generic tests
    @repo, @remote = setup_repo(:git)
    File.write(File.join(@repo, 'README.md'), 'test repo content')
  end

  def teardown
    cleanup_all_filesystems
    FileUtils.remove_entry(@test_dir) if @test_dir && File.exist?(@test_dir)
    FileUtils.remove_entry(@repo) if @repo && File.exist?(@repo)
    FileUtils.remove_entry(@remote) if @remote && File.exist?(@remote)
  end

  # === Generic test implementation ===

  private

  def create_test_provider
    Snapshot::CopyProvider.new(@repo)
  end

  def provider_skip_reason
    nil # Copy provider is always available
  end

  def expected_max_creation_time
    30.0 # Copy operations can be slower than CoW
  end

  def expected_max_cleanup_time
    10.0 # Cleanup involves removing files
  end

  def expected_concurrent_count
    5 # Copy provider handles concurrency well
  end

  def supports_space_efficiency_test?
    false # Copy provider doesn't support CoW
  end

  def create_workspace_destination(suffix = nil)
    base_name = suffix ? "copy_workspace_#{suffix}" : 'copy_workspace'
    Dir.mktmpdir(base_name)
  end

  def cleanup_test_workspace(workspace_dir)
    FileUtils.rm_rf(workspace_dir) if workspace_dir && File.exist?(workspace_dir)
  end

  def test_repo_content
    'test repo content'
  end

  def verify_cleanup_behavior(workspace_dir, _result_path)
    # For copy provider, cleanup should completely remove the workspace
    refute File.exist?(workspace_dir), 'Workspace directory should not exist after cleanup'
  end

  public

  # === Provider-specific tests ===

  def test_copy_provider_always_available
    Snapshot::CopyProvider.new('.')
    assert Snapshot::CopyProvider.available?('.')
  end

  def test_copy_provider_fallback_detection
    repo, remote = setup_repo(:git)

    # Stub all other providers to be unavailable
    Snapshot::ZfsProvider.stub(:available?, false) do
      Snapshot::BtrfsProvider.stub(:available?, false) do
        Snapshot::OverlayFsProvider.stub(:available?, false) do
          provider = Snapshot.provider_for(repo)
          assert_kind_of Snapshot::CopyProvider, provider
        end
      end
    end
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_copy_with_reflink_support_on_btrfs
    skip 'Copy reflink tests only run on Linux' unless linux?

    # Create Btrfs filesystem to test reflink support
    begin
      btrfs_mount = create_loop_filesystem('btrfs', 'reflink_test')
    rescue StandardError => e
      skip "Failed to create Btrfs filesystem for reflink test: #{e.message}"
    end

    repo_dir = File.join(btrfs_mount, 'repo')
    FileUtils.mkdir_p(repo_dir)

    # Create test repository content
    File.write(File.join(repo_dir, 'README.md'), 'test repo content')
    large_content = 'x' * (1024 * 1024) # 1MB file
    File.write(File.join(repo_dir, 'large_file.dat'), large_content)

    provider = Snapshot::CopyProvider.new(repo_dir)
    workspace_dir = File.join(btrfs_mount, 'workspace')

    begin
      # Measure space before copy
      space_before = get_filesystem_used_space(btrfs_mount)

      # Create workspace
      start_time = Time.now
      result_path = provider.create_workspace(workspace_dir)
      creation_time = Time.now - start_time

      # Measure space after copy
      space_after = get_filesystem_used_space(btrfs_mount)
      space_used = space_after - space_before

      # Verify workspace was created
      assert File.exist?(File.join(result_path, 'README.md'))
      assert File.exist?(File.join(result_path, 'large_file.dat'))

      # With reflinks, space usage should be minimal (much less than file size)
      assert space_used < 512 * 1024, "Copy used #{space_used} bytes, expected < 512KB with reflinks"

      # Copy should be fast with reflinks
      assert creation_time < 2.0, "Copy took #{creation_time}s, expected < 2s with reflinks"

      # Verify CoW behavior - modifying copied file shouldn't affect original
      File.write(File.join(result_path, 'large_file.dat'), 'modified content')
      assert_equal large_content, File.read(File.join(repo_dir, 'large_file.dat'))
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
    end
  end

  def test_copy_fallback_on_ext4_without_reflinks
    skip 'Copy fallback tests only run on Linux' unless linux?

    # Create ext4 filesystem (typically no reflink support)
    begin
      ext4_mount = create_loop_filesystem('ext4', 'fallback_test')
    rescue StandardError => e
      skip "Failed to create ext4 filesystem for fallback test: #{e.message}"
    end

    repo_dir = File.join(ext4_mount, 'repo')
    FileUtils.mkdir_p(repo_dir)

    # Create test repository content
    File.write(File.join(repo_dir, 'README.md'), 'test repo content')
    medium_content = 'y' * (100 * 1024) # 100KB file
    File.write(File.join(repo_dir, 'medium_file.dat'), medium_content)

    provider = Snapshot::CopyProvider.new(repo_dir)
    workspace_dir = File.join(ext4_mount, 'workspace')

    begin
      # Measure space before copy
      space_before = get_filesystem_used_space(ext4_mount)

      # Create workspace
      Time.now
      result_path = provider.create_workspace(workspace_dir)
      Time.now

      # Measure space after copy
      space_after = get_filesystem_used_space(ext4_mount)
      space_used = space_after - space_before

      # Verify workspace was created
      assert File.exist?(File.join(result_path, 'README.md'))
      assert File.exist?(File.join(result_path, 'medium_file.dat'))

      # Without reflinks, space usage should be roughly double the original
      original_size = File.size(File.join(repo_dir, 'medium_file.dat'))
      assert space_used > original_size * 0.8,
             "Expected regular copy to use ~#{original_size} bytes, used #{space_used}"
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
    end
  end

  def test_copy_cross_platform_behavior
    repo, remote = setup_repo(:git)

    # Create diverse content to test cross-platform handling
    File.write(File.join(repo, 'text_file.txt'), 'text content')
    File.write(File.join(repo, 'binary_file.bin'), "\x00\x01\x02\x03")
    FileUtils.mkdir_p(File.join(repo, 'subdir'))
    File.write(File.join(repo, 'subdir', 'nested_file.txt'), 'nested content')

    provider = Snapshot::CopyProvider.new(repo)
    workspace_dir = Dir.mktmpdir('cross_platform_test')

    begin
      result_path = provider.create_workspace(workspace_dir)

      # Verify all content was copied correctly
      assert File.exist?(File.join(result_path, 'text_file.txt'))
      assert File.exist?(File.join(result_path, 'binary_file.bin'))
      assert File.exist?(File.join(result_path, 'subdir', 'nested_file.txt'))

      # Verify content integrity
      assert_equal 'text content', File.read(File.join(result_path, 'text_file.txt'))
      assert_equal "\x00\x01\x02\x03", File.read(File.join(result_path, 'binary_file.bin'))
      assert_equal 'nested content', File.read(File.join(result_path, 'subdir', 'nested_file.txt'))

      # Verify isolation - changes don't affect original
      File.write(File.join(result_path, 'new_file.txt'), 'new content')
      refute File.exist?(File.join(repo, 'new_file.txt'))
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_copy_error_conditions
    repo, remote = setup_repo(:git)
    provider = Snapshot::CopyProvider.new(repo)

    # Test with invalid destination permissions
    invalid_dest = '/root/invalid_copy_test'
    assert_raises(Errno::EACCES, RuntimeError) do
      provider.create_workspace(invalid_dest)
    end

    # Test cleanup of non-existent workspace
    begin
      provider.cleanup_workspace('/non/existent/path')
      # Should not raise any exception
    rescue StandardError => e
      flunk "Cleanup should not raise exception: #{e.message}"
    end
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_copy_performance_with_large_repositories
    repo, remote = setup_repo(:git)

    # Create a large repository
    FileUtils.mkdir_p(File.join(repo, 'large_dir'))
    500.times do |i|
      File.write(File.join(repo, 'large_dir', "file_#{i}.txt"), "content #{i}" * 50)
    end

    provider = Snapshot::CopyProvider.new(repo)
    workspace_dir = Dir.mktmpdir('large_copy_test')

    begin
      # Measure performance
      start_time = Time.now
      result_path = provider.create_workspace(workspace_dir)
      creation_time = Time.now - start_time

      # Should complete in reasonable time even for large repos
      assert creation_time < 30.0, "Large copy took #{creation_time}s, expected < 30s"

      # Verify all files were copied
      assert File.exist?(File.join(result_path, 'large_dir', 'file_0.txt'))
      assert File.exist?(File.join(result_path, 'large_dir', 'file_499.txt'))

      # Test cleanup performance
      start_time = Time.now
      provider.cleanup_workspace(workspace_dir)
      cleanup_time = Time.now - start_time

      assert cleanup_time < 10.0, "Large cleanup took #{cleanup_time}s, expected < 10s"
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_copy_concurrent_operations
    repo, remote = setup_repo(:git)

    # Add some content
    File.write(File.join(repo, 'shared_file.txt'), 'shared content')

    provider = Snapshot::CopyProvider.new(repo)
    workspaces = []
    threads = []

    begin
      # Create multiple workspaces concurrently
      5.times do |i|
        threads << Thread.new do
          workspace_dir = Dir.mktmpdir("concurrent_copy_#{i}")
          workspaces << workspace_dir
          result_path = provider.create_workspace(workspace_dir)

          # Each thread modifies its own workspace
          File.write(File.join(result_path, "thread_#{i}.txt"), "thread #{i} content")
          sleep(0.1) # Simulate some work
        end
      end

      # Wait for all threads
      threads.each(&:join)

      # Verify all workspaces were created successfully and independently
      workspaces.each_with_index do |ws, i|
        assert File.exist?(File.join(ws, 'README.md'))
        assert File.exist?(File.join(ws, 'shared_file.txt'))
        assert File.exist?(File.join(ws, "thread_#{i}.txt"))

        # Verify other threads' files don't exist
        (0...5).each do |j|
          next if j == i

          refute File.exist?(File.join(ws, "thread_#{j}.txt"))
        end
      end
    ensure
      # Cleanup all workspaces
      workspaces.each do |ws|
        provider.cleanup_workspace(ws) if File.exist?(ws)
      end
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end
end
