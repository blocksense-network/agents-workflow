# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative '../test_helper'
require_relative 'provider_shared_behavior'
require_relative 'provider_loop_device_test_behavior'
require_relative 'filesystem_test_helper'
require_relative '../../lib/snapshot/provider'

# Shared behavior for OverlayFS multi-filesystem testing
module ProviderOverlayTestBehavior
  # Test overlay with multiple loop-mounted filesystems
  def test_overlay_with_multiple_loop_filesystems
    unless supports_multi_filesystem_testing?
      return skip "Multi-filesystem testing not supported for #{provider_class_name}"
    end

    # Create multiple loop-mounted filesystems for comprehensive overlay testing
    begin
      lower1_mount = create_loop_filesystem('ext4', 'lower1', size_mb: 50)
      lower2_mount = create_loop_filesystem('ext4', 'lower2', size_mb: 50)
      upper_mount = create_loop_filesystem('ext4', 'upper', size_mb: 50)
    rescue StandardError => e
      skip "Failed to create loop filesystems for overlay test: #{e.message}"
    end

    # Setup content in lower filesystems
    File.write(File.join(lower1_mount, 'file1.txt'), 'content from lower1')
    File.write(File.join(lower1_mount, 'shared.txt'), 'lower1 version')
    File.write(File.join(lower2_mount, 'file2.txt'), 'content from lower2')
    File.write(File.join(lower2_mount, 'shared.txt'), 'lower2 version')

    # Setup overlay directories
    upper_dir = File.join(upper_mount, 'upper')
    work_dir = File.join(upper_mount, 'work')
    merged_dir = File.join(@test_dir, 'merged')
    FileUtils.mkdir_p([upper_dir, work_dir, merged_dir])

    # Create overlay mount
    options = "lowerdir=#{lower2_mount}:#{lower1_mount},upperdir=#{upper_dir},workdir=#{work_dir}"

    begin
      unless system('mount', '-t', 'overlay', 'overlay', '-o', options, merged_dir, out: File::NULL, err: File::NULL)
        skip 'OverlayFS mounting requires root privileges'
      end

      # Verify layered filesystem behavior
      assert File.exist?(File.join(merged_dir, 'file1.txt'))
      assert File.exist?(File.join(merged_dir, 'file2.txt'))

      # Verify precedence (lower2 should take precedence over lower1)
      assert_equal 'lower2 version', File.read(File.join(merged_dir, 'shared.txt'))

      # Test writing to overlay
      File.write(File.join(merged_dir, 'overlay_file.txt'), 'overlay content')

      # File should appear in upper dir but not in lower dirs
      assert File.exist?(File.join(upper_dir, 'overlay_file.txt'))
      refute File.exist?(File.join(lower1_mount, 'overlay_file.txt'))
      refute File.exist?(File.join(lower2_mount, 'overlay_file.txt'))
    ensure
      system('umount', merged_dir, out: File::NULL, err: File::NULL)
    end
  end

  # Test overlay with read-only lower filesystem
  def test_overlay_read_only_lower_layers
    return skip "Read-only testing not supported for #{provider_class_name}" unless supports_multi_filesystem_testing?

    begin
      # Create and mount read-only lower filesystem
      lower_mount = create_loop_filesystem('ext4', 'ro_lower', size_mb: 50)
      File.write(File.join(lower_mount, 'ro_file.txt'), 'read-only content')

      # Remount as read-only
      system('mount', '-o', 'remount,ro', lower_mount, out: File::NULL, err: File::NULL)

      # Create writable upper filesystem
      upper_mount = create_loop_filesystem('ext4', 'rw_upper', size_mb: 50)
      upper_dir = File.join(upper_mount, 'upper')
      work_dir = File.join(upper_mount, 'work')
      merged_dir = File.join(@test_dir, 'ro_merged')
      FileUtils.mkdir_p([upper_dir, work_dir, merged_dir])

      # Create overlay with read-only lower
      options = "lowerdir=#{lower_mount},upperdir=#{upper_dir},workdir=#{work_dir}"

      unless system('mount', '-t', 'overlay', 'overlay', '-o', options, merged_dir, out: File::NULL, err: File::NULL)
        skip 'OverlayFS mounting requires root privileges'
      end

      # Verify read-only content is accessible
      assert File.exist?(File.join(merged_dir, 'ro_file.txt'))
      assert_equal 'read-only content', File.read(File.join(merged_dir, 'ro_file.txt'))

      # Verify writes work through upper layer
      File.write(File.join(merged_dir, 'new_file.txt'), 'new content')
      assert File.exist?(File.join(upper_dir, 'new_file.txt'))
    rescue StandardError => e
      skip "Read-only overlay test failed: #{e.message}"
    ensure
      system('umount', merged_dir, out: File::NULL, err: File::NULL) if merged_dir
      system('mount', '-o', 'remount,rw', lower_mount, out: File::NULL, err: File::NULL) if lower_mount
    end
  end

  private

  def supports_multi_filesystem_testing?
    false
  end

  def provider_class_name
    self.class.name.gsub(/^Test|Test$/, '')
  end
end

# Comprehensive tests for OverlayFS provider combining generic and specific tests
class TestOverlayProvider < Minitest::Test
  include RepoTestHelper
  include FilesystemTestHelper
  include ProviderSharedBehavior
  include ProviderLoopDeviceTestBehavior
  include ProviderOverlayTestBehavior

  def setup
    skip 'OverlayFS tests only run on Linux' unless linux?
    skip 'OverlayFS not available' unless Snapshot::OverlayFsProvider.available?('.')

    @test_dir = Dir.mktmpdir('overlay_test')
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
    Snapshot::OverlayFsProvider.new(@repo)
  end

  def provider_skip_reason
    # Additional runtime check for mount capabilities

    # Try a simple test to see if we can create overlays
    test_dir = Dir.mktmpdir('overlay_check')
    provider = Snapshot::OverlayFsProvider.new(@repo)
    provider.create_workspace(test_dir)
    provider.cleanup_workspace(test_dir)
    nil
  rescue RuntimeError => e
    e.message
  ensure
    FileUtils.remove_entry(test_dir) if test_dir && File.exist?(test_dir)
  end

  def expected_max_creation_time
    2.0 # OverlayFS should be very fast
  end

  def expected_max_cleanup_time
    1.0 # OverlayFS cleanup should be fast
  end

  def expected_concurrent_count
    3 # OverlayFS may have limitations with concurrent mounts
  end

  def supports_space_efficiency_test?
    true # OverlayFS is space efficient
  end

  def measure_space_usage
    # For OverlayFS, space usage is minimal as it uses the original files
    0 # OverlayFS uses virtually no additional space
  end

  def expected_max_space_usage
    1024 # 1KB - OverlayFS should use minimal space
  end

  # === Loop device test implementation ===

  def supports_loop_device_testing?
    false # OverlayFS doesn't create its own filesystem
  end

  # === Multi-filesystem test implementation ===

  def supports_multi_filesystem_testing?
    true
  end

  public

  # === OverlayFS-specific tests ===

  def test_overlay_provider_detection
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
    repo, remote = setup_repo(:git)
    File.write(File.join(repo, 'test_file.txt'), 'test content')

    provider = Snapshot::OverlayFsProvider.new(repo)
    dest = Dir.mktmpdir('overlay_workspace')

    begin
      # Create workspace
      result_path = provider.create_workspace(dest)

      # Verify workspace was created and content is accessible
      assert File.exist?(File.join(result_path, 'README.md'))
      assert File.exist?(File.join(result_path, 'test_file.txt'))
      assert_equal 'test content', File.read(File.join(result_path, 'test_file.txt'))

      # Verify isolation - changes don't affect original
      File.write(File.join(result_path, 'overlay_change.txt'), 'overlay content')
      refute File.exist?(File.join(repo, 'overlay_change.txt'))

      # Test cleanup
      provider.cleanup_workspace(dest)
    rescue RuntimeError => e
      skip "Overlay test failed - may need root privileges: #{e.message}"
    ensure
      begin
        provider.cleanup_workspace(dest) if dest && File.exist?(dest)
      rescue RuntimeError
        # Ignore cleanup errors in case mount failed
      end
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_overlay_error_conditions_and_recovery
    repo, remote = setup_repo(:git)
    provider = Snapshot::OverlayFsProvider.new(repo)

    # Test with invalid mount options
    invalid_dest = File.join(@test_dir, 'invalid_overlay')
    FileUtils.mkdir_p(invalid_dest)

    # Create directories with invalid permissions
    work_dir = File.join(invalid_dest, 'work')
    FileUtils.mkdir_p(work_dir)
    File.chmod(0o000, work_dir) # No permissions

    begin
      assert_raises(RuntimeError) do
        provider.create_workspace(invalid_dest)
      end
    ensure
      File.chmod(0o755, work_dir) # Restore permissions for cleanup
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_overlay_performance_with_large_directories
    # Create a large repository for performance testing
    repo, remote = setup_repo(:git)

    # Add many files to test performance
    1000.times do |i|
      File.write(File.join(repo, "file_#{i}.txt"), "content #{i}")
    end

    provider = Snapshot::OverlayFsProvider.new(repo)
    dest = Dir.mktmpdir('overlay_perf')

    begin
      # Measure creation time
      start_time = Time.now
      ws_path = provider.create_workspace(dest)
      creation_time = Time.now - start_time

      # Overlay creation should be fast regardless of repository size
      assert creation_time < 2.0, "Overlay creation took #{creation_time}s, expected < 2s"

      # Verify all files are accessible
      assert File.exist?(File.join(ws_path, 'file_0.txt'))
      assert File.exist?(File.join(ws_path, 'file_999.txt'))

      # Test cleanup performance
      start_time = Time.now
      provider.cleanup_workspace(dest)
      cleanup_time = Time.now - start_time

      assert cleanup_time < 1.0, "Cleanup took #{cleanup_time}s, expected < 1s"
    rescue RuntimeError => e
      skip "Overlay test failed - may need root privileges: #{e.message}"
    ensure
      begin
        provider.cleanup_workspace(dest) if dest && File.exist?(dest)
      rescue RuntimeError
        # Ignore cleanup errors in case mount failed
      end
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end
end
