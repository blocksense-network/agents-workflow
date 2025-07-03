# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative '../test_helper'
require_relative 'provider_shared_behavior'
require_relative 'provider_quota_test_behavior'
require_relative 'provider_loop_device_test_behavior'
require_relative '../../lib/snapshot/provider'

# Comprehensive tests for ZFS provider combining generic and specific tests
class TestZfsProvider < Minitest::Test
  include RepoTestHelper
  include ProviderSharedBehavior
  include ProviderQuotaTestBehavior
  include ProviderLoopDeviceTestBehavior

  def setup
    skip 'ZFS tests only run on Linux' unless linux?
    skip 'ZFS tools not available' unless system('which', 'zfs', out: File::NULL, err: File::NULL)

    @test_dir = Dir.mktmpdir('zfs_test')
    @pool_name = "test_pool_#{Process.pid}_#{Time.now.to_i}"
    @image_file = File.join(@test_dir, 'zfs_test.img')
    @repo_dir = nil
    @pool_created = false

    begin
      create_zfs_pool_and_repo
    rescue StandardError => e
      skip "Failed to create ZFS test environment: #{e.message}"
    end
  end

  def teardown
    cleanup_zfs_pool if @pool_created
    FileUtils.remove_entry(@test_dir) if @test_dir && File.exist?(@test_dir)
  end

  # === Generic test implementation ===

  private

  def create_test_provider
    Snapshot::ZfsProvider.new(@repo_dir)
  end

  def provider_skip_reason
    return 'ZFS pool not created' unless @pool_created

    nil
  end

  def expected_max_creation_time
    5.0 # ZFS snapshots should be very fast
  end

  def expected_max_cleanup_time
    3.0 # ZFS cleanup should be fast
  end

  def expected_concurrent_count
    5 # ZFS handles concurrency well
  end

  def supports_space_efficiency_test?
    true # ZFS supports CoW
  end

  def measure_space_usage
    pool_used_space
  end

  def expected_max_space_usage
    1024 * 1024 # 1MB for ZFS metadata
  end

  def create_workspace_destination(suffix = nil)
    base_name = suffix ? "zfs_workspace_#{suffix}" : 'zfs_workspace'
    Dir.mktmpdir(base_name)
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
    Snapshot::ZfsProvider
  end

  def expected_native_creation_time
    5.0 # ZFS snapshots are fast
  end

  def expected_native_cleanup_time
    3.0 # ZFS cleanup is fast
  end

  # === Quota test implementation ===

  def supports_quota_testing?
    true
  end

  def setup_quota_environment
    # Set a quota on the dataset
    dataset = get_dataset_for_path(@repo_dir)
    system('zfs', 'set', 'quota=10M', dataset, out: File::NULL, err: File::NULL)
  end

  def cleanup_quota_environment
    # Quota cleanup handled by pool destruction
  end

  def verify_quota_behavior(quota_exceeded)
    # ZFS should enforce quotas strictly
    assert quota_exceeded, 'ZFS should have enforced the 10MB quota limit'
  end

  public

  # === ZFS-specific tests ===

  def test_zfs_snapshot_and_clone_operations
    provider = Snapshot::ZfsProvider.new(@repo_dir)
    workspace_dir = Dir.mktmpdir('zfs_workspace')

    begin
      # Create workspace using ZFS snapshot/clone
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

      # Test performance - snapshot creation should be fast (< 5 seconds for small repos)
      assert creation_time < 5.0, "Snapshot creation took #{creation_time}s, expected < 5s"

      # Test cleanup
      start_time = Time.now
      provider.cleanup_workspace(workspace_dir)
      cleanup_time = Time.now - start_time

      # Verify cleanup performance
      assert cleanup_time < 3.0, "Cleanup took #{cleanup_time}s, expected < 3s"
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
      FileUtils.rm_rf(workspace_dir)
    end
  end

  def test_zfs_error_conditions
    provider = Snapshot::ZfsProvider.new(@repo_dir)

    # Test with invalid destination
    assert_raises(RuntimeError) do
      provider.create_workspace('/invalid/readonly/path')
    end

    # Test cleanup of non-existent workspace
    assert_nothing_raised do
      provider.cleanup_workspace('/non/existent/path')
    end
  end

  def test_zfs_space_usage_efficiency
    provider = Snapshot::ZfsProvider.new(@repo_dir)
    workspace_dir = Dir.mktmpdir('zfs_space_test')

    begin
      # Measure space before snapshot
      space_before = pool_used_space

      # Create workspace
      provider.create_workspace(workspace_dir)

      # Measure space after snapshot (should be minimal due to CoW)
      space_after = pool_used_space
      space_used = space_after - space_before

      # Snapshot should use minimal space (less than 1MB for metadata)
      assert space_used < 1024 * 1024, "Snapshot used #{space_used} bytes, expected < 1MB"
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
      FileUtils.rm_rf(workspace_dir)
    end
  end

  private

  # ZFS-specific helper methods

  def create_zfs_pool_and_repo(pool_size: '100M')
    # Create loop device image file
    system('dd', 'if=/dev/zero', "of=#{@image_file}", 'bs=1M', "count=#{pool_size.to_i}",
           out: File::NULL, err: File::NULL)

    # Create ZFS pool on loop device
    unless system('zpool', 'create', @pool_name, @image_file, out: File::NULL, err: File::NULL)
      raise 'Failed to create ZFS pool - may need root privileges'
    end

    @pool_created = true

    # Create dataset and mount point
    @repo_dir = File.join(@test_dir, 'repo')
    dataset = "#{@pool_name}/repo"
    system('zfs', 'create', '-o', "mountpoint=#{@repo_dir}", dataset)

    # Initialize test repository content
    File.write(File.join(@repo_dir, 'README.md'), 'test repo content')
    File.write(File.join(@repo_dir, 'test_file.txt'), 'additional content')
  end

  def cleanup_zfs_pool
    return unless @pool_created

    # Destroy pool (this also destroys all datasets)
    system('zpool', 'destroy', @pool_name, out: File::NULL, err: File::NULL)
    @pool_created = false
  end

  def get_dataset_for_path(path)
    # Get the ZFS dataset that contains the given path
    output = `zfs list -H -o name,mountpoint 2>/dev/null`
    output.lines.each do |line|
      name, mountpoint = line.strip.split("\t")
      return name if path.start_with?(mountpoint)
    end
    nil
  end

  def pool_used_space
    output = `zpool list -H -o used #{@pool_name} 2>/dev/null`.strip
    return 0 if output.empty?

    # Convert output to bytes (handles K, M, G suffixes)
    case output
    when /(\d+(?:\.\d+)?)K/
      (::Regexp.last_match(1).to_f * 1024).to_i
    when /(\d+(?:\.\d+)?)M/
      (::Regexp.last_match(1).to_f * 1024 * 1024).to_i
    when /(\d+(?:\.\d+)?)G/
      (::Regexp.last_match(1).to_f * 1024 * 1024 * 1024).to_i
    else
      output.to_i
    end
  end
end
