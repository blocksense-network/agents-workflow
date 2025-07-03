# frozen_string_literal: true

# Shared behavior for providers that need loop device filesystem testing
module ProviderLoopDeviceTestBehavior
  # Test provider detection on its native filesystem
  def test_provider_detection_on_native_filesystem
    return skip "Loop device testing not supported for #{provider_class_name}" unless supports_loop_device_testing?

    setup_loop_device_environment

    begin
      provider = Snapshot.provider_for(@repo_dir || @repo)
      assert_kind_of expected_provider_class, provider
    ensure
      cleanup_loop_device_environment
    end
  end

  # Test provider-specific filesystem operations
  def test_provider_native_filesystem_operations
    return skip "Loop device testing not supported for #{provider_class_name}" unless supports_loop_device_testing?

    setup_loop_device_environment
    provider = create_test_provider
    workspace_dir = create_native_workspace_destination

    begin
      # Create workspace using native filesystem operations
      start_time = Time.now
      result_path = provider.create_workspace(workspace_dir)
      creation_time = Time.now - start_time

      # Verify workspace was created
      assert File.exist?(result_path)
      assert File.exist?(File.join(result_path, 'README.md'))

      # Verify CoW behavior - changes in workspace don't affect original
      File.write(File.join(result_path, 'workspace_file.txt'), 'workspace content')
      refute File.exist?(File.join(@repo_dir || @repo, 'workspace_file.txt'))

      # Verify original file content is accessible
      assert_equal 'test repo content', File.read(File.join(result_path, 'README.md'))

      # Test performance - should be fast for native operations
      max_creation_time = expected_native_creation_time
      assert creation_time < max_creation_time,
             "Native operation took #{creation_time}s, expected < #{max_creation_time}s"

      # Test cleanup
      start_time = Time.now
      provider.cleanup_workspace(workspace_dir)
      cleanup_time = Time.now - start_time

      max_cleanup_time = expected_native_cleanup_time
      assert cleanup_time < max_cleanup_time,
             "Native cleanup took #{cleanup_time}s, expected < #{max_cleanup_time}s"
    ensure
      cleanup_loop_device_environment
    end
  end

  private

  # Abstract methods for loop device testing
  def supports_loop_device_testing?
    false
  end

  def setup_loop_device_environment
    # Override in subclasses that support loop device testing
  end

  def cleanup_loop_device_environment
    # Override in subclasses that support loop device testing
  end

  def expected_provider_class
    raise NotImplementedError, 'Subclass must implement expected_provider_class'
  end

  def create_native_workspace_destination
    # Override in subclasses for provider-specific workspace paths
    create_workspace_destination('native')
  end

  def expected_native_creation_time
    5.0 # seconds - default for native operations
  end

  def expected_native_cleanup_time
    3.0 # seconds - default for native cleanup
  end

  def provider_class_name
    self.class.name.gsub(/^Test|Test$/, '')
  end
end
