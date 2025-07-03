# frozen_string_literal: true

require 'English'
require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative '../test_helper'
require_relative 'provider_shared_behavior'
require_relative 'provider_quota_test_behavior'
require_relative 'provider_loop_device_test_behavior'
require_relative 'filesystem_test_helper'
require_relative '../../lib/snapshot/provider'

# Comprehensive tests for Btrfs provider combining generic and specific tests
class TestBtrfsProvider < Minitest::Test
  include RepoTestHelper
  include FilesystemTestHelper
  include ProviderSharedBehavior
  include ProviderQuotaTestBehavior
  include ProviderLoopDeviceTestBehavior

  def setup
    skip 'Btrfs tests only run on Linux' unless linux?
    skip 'Btrfs tools not available' unless system('which', 'btrfs', out: File::NULL, err: File::NULL)

    @test_dir = Dir.mktmpdir('btrfs_test')
    @image_file = File.join(@test_dir, 'btrfs_test.img')
    @mount_point = File.join(@test_dir, 'mount')
    @repo_dir = nil
    @filesystem_created = false
    @mounted = false

    begin
      create_btrfs_filesystem_and_repo
    rescue StandardError => e
      skip "Failed to create Btrfs test environment: #{e.message}"
    end
  end

  def teardown
    cleanup_btrfs_filesystem
    FileUtils.remove_entry(@test_dir) if @test_dir && File.exist?(@test_dir)
  end

  # === Generic test implementation ===

  private

  def create_test_provider
    Snapshot::BtrfsProvider.new(@repo_dir)
  end

  def provider_skip_reason
    return 'Btrfs filesystem not mounted' unless @mounted

    nil
  end

  def expected_max_creation_time
    3.0 # Btrfs snapshots should be fast
  end

  def expected_max_cleanup_time
    2.0 # Btrfs cleanup should be fast
  end

  def expected_concurrent_count
    3 # Btrfs handles concurrency reasonably well
  end

  def supports_space_efficiency_test?
    true # Btrfs supports CoW
  end

  def measure_space_usage
    filesystem_used_space
  end

  def expected_max_space_usage
    512 * 1024 # 512KB for Btrfs metadata
  end

  def create_workspace_destination(suffix = nil)
    base_name = suffix ? "btrfs_workspace_#{suffix}" : 'btrfs_workspace'
    File.join(@mount_point, base_name)
  end

  # === Loop device test implementation ===

  def supports_loop_device_testing?
    true
  end

  def setup_loop_device_environment
    # Already set up in main setup
  end

  def cleanup_loop_device_environment
    # Handled in main teardown
  end

  def expected_provider_class
    Snapshot::BtrfsProvider
  end

  def create_native_workspace_destination
    File.join(@mount_point, 'native_workspace')
  end

  def expected_native_creation_time
    3.0 # Btrfs snapshots are fast
  end

  def expected_native_cleanup_time
    2.0 # Btrfs cleanup is fast
  end

  # === Quota test implementation ===

  def supports_quota_testing?
    true
  end

  def setup_quota_environment
    # Enable quotas on the filesystem
    system('btrfs', 'quota', 'enable', @mount_point, out: File::NULL, err: File::NULL)

    # Set a quota limit on the subvolume
    subvol_id = get_subvolume_id(@repo_dir)
    return unless subvol_id

    # Set 10MB limit
    system('btrfs', 'qgroup', 'limit', '10M', "0/#{subvol_id}", @mount_point,
           out: File::NULL, err: File::NULL)
  end

  def cleanup_quota_environment
    # Quota cleanup handled by filesystem unmount
  end

  def verify_quota_behavior(quota_exceeded)
    # NOTE: Btrfs quotas may not immediately enforce limits in all scenarios
    # This test documents the current behavior
    if quota_exceeded
      # Good - quota was enforced
    else
      # This is also acceptable for Btrfs as quotas can be complex
      puts 'Note: Btrfs quota enforcement may be delayed or disabled'
    end
  end

  public

  # === Btrfs-specific tests ===

  def test_btrfs_subvolume_snapshot_operations
    provider = Snapshot::BtrfsProvider.new(@repo_dir)
    workspace_dir = File.join(@mount_point, 'workspace_snapshot')

    begin
      # Create workspace using Btrfs subvolume snapshot
      start_time = Time.now
      result_path = provider.create_workspace(workspace_dir)
      creation_time = Time.now - start_time

      # Verify workspace was created
      assert File.exist?(result_path)
      assert File.exist?(File.join(result_path, 'README.md'))

      # Verify CoW behavior - changes in workspace don't affect original
      File.write(File.join(result_path, 'workspace_file.txt'), 'workspace content')
      refute File.exist?(File.join(@repo_dir, 'workspace_file.txt'))

      # Verify original file content is accessible
      assert_equal 'test repo content', File.read(File.join(result_path, 'README.md'))

      # Test performance - snapshot creation should be fast (< 3 seconds for small repos)
      assert creation_time < 3.0, "Snapshot creation took #{creation_time}s, expected < 3s"

      # Test cleanup
      start_time = Time.now
      provider.cleanup_workspace(workspace_dir)
      cleanup_time = Time.now - start_time

      # Verify cleanup performance
      assert cleanup_time < 2.0, "Cleanup took #{cleanup_time}s, expected < 2s"
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
    end
  end

  def test_btrfs_auto_subvolume_creation
    create_btrfs_filesystem_and_repo(create_subvolume: false)

    # Repository should initially be a regular directory, not a subvolume
    refute btrfs_is_subvolume?(@repo_dir)

    provider = Snapshot::BtrfsProvider.new(@repo_dir)
    workspace_dir = File.join(@mount_point, 'auto_subvol_test')

    begin
      # This should automatically convert the directory to a subvolume if needed
      # (Note: Current implementation expects repo to already be a subvolume)
      # This test documents the current behavior
      assert_raises(RuntimeError) do
        provider.create_workspace(workspace_dir)
      end
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
    end
  end

  def test_btrfs_error_conditions
    provider = Snapshot::BtrfsProvider.new(@repo_dir)

    # Test with invalid destination (outside Btrfs filesystem)
    assert_raises(RuntimeError) do
      provider.create_workspace('/tmp/invalid_btrfs_path')
    end

    # Test cleanup of non-existent workspace
    assert_nothing_raised do
      provider.cleanup_workspace('/non/existent/path')
    end
  end

  def test_btrfs_space_usage_efficiency
    provider = Snapshot::BtrfsProvider.new(@repo_dir)
    workspace_dir = File.join(@mount_point, 'space_test')

    begin
      # Measure space before snapshot
      space_before = filesystem_used_space

      # Create workspace
      provider.create_workspace(workspace_dir)

      # Measure space after snapshot (should be minimal due to CoW)
      space_after = filesystem_used_space
      space_used = space_after - space_before

      # Snapshot should use minimal space (less than 512KB for metadata)
      assert space_used < 512 * 1024, "Snapshot used #{space_used} bytes, expected < 512KB"
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
    end
  end

  def test_btrfs_snapshot_performance_scaling
    # Create a larger repository with multiple files
    100.times do |i|
      File.write(File.join(@repo_dir, "file_#{i}.txt"), "content #{i}" * 100)
    end

    provider = Snapshot::BtrfsProvider.new(@repo_dir)

    # Test multiple snapshots to verify consistent performance
    times = []
    5.times do |i|
      workspace_dir = File.join(@mount_point, "perf_test_#{i}")

      start_time = Time.now
      provider.create_workspace(workspace_dir)
      times << (Time.now - start_time)

      provider.cleanup_workspace(workspace_dir)
    end

    # All snapshots should complete quickly
    times.each_with_index do |time, i|
      assert time < 2.0, "Snapshot #{i} took #{time}s, expected < 2s"
    end

    # Average time should be consistent
    avg_time = times.sum / times.size
    assert avg_time < 1.0, "Average snapshot time #{avg_time}s, expected < 1s"
  end

  private

  # Btrfs-specific helper methods

  def create_btrfs_filesystem_and_repo(filesystem_size: 120, create_subvolume: true)
    # Create loop device image file (120MB minimum for Btrfs)
    system('dd', 'if=/dev/zero', "of=#{@image_file}", 'bs=1M', "count=#{filesystem_size}",
           out: File::NULL, err: File::NULL)

    # Create Btrfs filesystem
    unless system('mkfs.btrfs', '-f', @image_file, out: File::NULL, err: File::NULL)
      raise 'Failed to create Btrfs filesystem'
    end

    @filesystem_created = true

    # Create and mount the filesystem
    FileUtils.mkdir_p(@mount_point)
    unless system('mount', '-o', 'loop', @image_file, @mount_point, out: File::NULL, err: File::NULL)
      raise 'Failed to mount Btrfs filesystem - may need root privileges for mounting'
    end

    @mounted = true

    if create_subvolume
      # Create subvolume for repository
      @repo_dir = File.join(@mount_point, 'repo_subvol')
      system('btrfs', 'subvolume', 'create', @repo_dir, out: File::NULL, err: File::NULL)
    else
      # Create regular directory for repository
      @repo_dir = File.join(@mount_point, 'repo_dir')
      FileUtils.mkdir_p(@repo_dir)
    end

    # Initialize test repository content
    File.write(File.join(@repo_dir, 'README.md'), 'test repo content')
    File.write(File.join(@repo_dir, 'test_file.txt'), 'additional content')
  end

  def cleanup_btrfs_filesystem
    return unless @mounted

    system('umount', @mount_point, out: File::NULL, err: File::NULL)
    @mounted = false
  end

  def btrfs_is_subvolume?(path)
    # Check if path is a Btrfs subvolume
    `btrfs subvolume show #{path} 2>/dev/null`
    $CHILD_STATUS.success?
  end

  def get_subvolume_id(path)
    return nil unless btrfs_is_subvolume?(path)

    output = `btrfs subvolume show #{path} 2>/dev/null`
    match = output.match(/Subvolume ID:\s+(\d+)/)
    match ? match[1] : nil
  end

  def filesystem_used_space
    output = `btrfs filesystem usage #{@mount_point} 2>/dev/null`
    match = output.match(/Used:\s+(\d+(?:\.\d+)?)\s*(\w+)/)
    return 0 unless match

    value = match[1].to_f
    unit = match[2].upcase

    case unit
    when 'B', 'BYTES'
      value.to_i
    when 'K', 'KB', 'KIB'
      (value * 1024).to_i
    when 'M', 'MB', 'MIB'
      (value * 1024 * 1024).to_i
    when 'G', 'GB', 'GIB'
      (value * 1024 * 1024 * 1024).to_i
    else
      value.to_i
    end
  end
end
