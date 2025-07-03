# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'

# Performance and concurrency test behaviors for providers
module ProviderPerformanceTestBehavior
  # Test performance characteristics
  def test_provider_performance
    provider = create_test_provider
    return skip provider_skip_reason if provider_skip_reason

    workspace_dir = create_workspace_destination

    begin
      # Measure creation time
      start_time = Time.now
      provider.create_workspace(workspace_dir)
      creation_time = Time.now - start_time

      # Verify reasonable performance (provider-specific expectations)
      max_creation_time = expected_max_creation_time
      assert creation_time < max_creation_time,
             "#{provider.class.name} creation took #{creation_time}s, expected < #{max_creation_time}s"

      # Measure cleanup time
      start_time = Time.now
      provider.cleanup_workspace(workspace_dir)
      cleanup_time = Time.now - start_time

      max_cleanup_time = expected_max_cleanup_time
      assert cleanup_time < max_cleanup_time,
             "#{provider.class.name} cleanup took #{cleanup_time}s, expected < #{max_cleanup_time}s"
    ensure
      cleanup_test_workspace(workspace_dir)
    end
  end

  # Test concurrent operations
  def test_provider_concurrent_operations
    provider = create_test_provider
    return skip provider_skip_reason if provider_skip_reason

    workspace_results = {}
    threads = []
    mutex = Mutex.new

    begin
      # Create multiple workspaces concurrently
      concurrent_count = expected_concurrent_count
      concurrent_count.times do |i|
        threads << Thread.new do
          workspace_dir = create_workspace_destination("concurrent_#{i}")
          result_path = provider.create_workspace(workspace_dir)

          # Simulate some work
          sleep(0.1)
          File.write(File.join(result_path, "thread_#{i}.txt"), "thread #{i} content")

          # Thread-safe storage of results
          mutex.synchronize { workspace_results[i] = workspace_dir }
        end
      end

      # Wait for all threads to complete
      threads.each(&:join)

      # Verify all workspaces were created successfully
      concurrent_count.times do |i|
        ws = workspace_results[i]
        assert ws, "Workspace #{i} should be recorded"
        assert File.exist?(File.join(ws, 'README.md')),
               "Concurrent workspace #{i} should contain README.md"
        assert File.exist?(File.join(ws, "thread_#{i}.txt")),
               "Concurrent workspace #{i} should contain its thread-specific file"
      end
    ensure
      # Cleanup all workspaces
      workspace_results.each_value do |ws|
        begin
          provider.cleanup_workspace(ws) if File.exist?(ws)
        rescue StandardError => e
          # Log but don't fail on cleanup errors
          puts "Warning: Failed to cleanup workspace #{ws}: #{e.message}"
        end
        cleanup_test_workspace(ws)
      end
    end
  end

  # Test space efficiency (for CoW providers)
  def test_provider_space_efficiency
    provider = create_test_provider
    return skip provider_skip_reason if provider_skip_reason
    return skip "Space efficiency test not applicable for #{provider.class.name}" unless supports_space_efficiency_test?

    workspace_dir = create_workspace_destination

    begin
      # Measure space before workspace creation
      space_before = measure_space_usage

      # Create workspace
      provider.create_workspace(workspace_dir)

      # Measure space after workspace creation
      space_after = measure_space_usage
      space_used = space_after - space_before

      # Verify space efficiency for CoW providers
      max_space_usage = expected_max_space_usage
      assert space_used < max_space_usage,
             "#{provider.class.name} used #{space_used} bytes, expected < #{max_space_usage} for CoW operation"
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
      cleanup_test_workspace(workspace_dir)
    end
  end
end
