# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'

# Core shared behaviors for basic provider operations
module ProviderCoreTestBehavior
  # Test basic workspace creation and isolation
  def test_provider_creates_isolated_workspace
    provider = create_test_provider
    return skip provider_skip_reason if provider_skip_reason

    workspace_dir = create_workspace_destination

    begin
      result_path = provider.create_workspace(workspace_dir)

      # Verify workspace was created and contains repo content
      assert File.exist?(result_path), "Workspace path should exist: #{result_path}"
      assert File.exist?(File.join(result_path, 'README.md')), 'Workspace should contain README.md'
      assert_equal test_repo_content, File.read(File.join(result_path, 'README.md'))

      # Verify isolation - changes in workspace don't affect original
      File.write(File.join(result_path, 'workspace_only_file.txt'), 'workspace content')
      refute File.exist?(File.join(provider.repo_path, 'workspace_only_file.txt')),
             'Changes in workspace should not affect original repo'
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
      cleanup_test_workspace(workspace_dir)
    end
  end

  # Test workspace cleanup behavior
  def test_provider_cleanup_workspace
    provider = create_test_provider
    return skip provider_skip_reason if provider_skip_reason

    workspace_dir = create_workspace_destination

    begin
      result_path = provider.create_workspace(workspace_dir)

      # Verify workspace exists
      assert File.exist?(result_path), 'Workspace should be created'

      # Cleanup workspace
      provider.cleanup_workspace(workspace_dir)

      # Verify cleanup behavior (implementation-specific)
      # Some providers may remove the directory, others may just unmount
      verify_cleanup_behavior(workspace_dir, result_path)
    ensure
      # Fallback cleanup
      cleanup_test_workspace(workspace_dir)
    end
  end

  # Test error handling for invalid destinations
  def test_provider_error_handling
    provider = create_test_provider
    return skip provider_skip_reason if provider_skip_reason

    # Test with invalid destination
    invalid_dest = '/root/invalid_provider_test'
    assert_raises(RuntimeError, Errno::EACCES, Errno::EROFS) do
      provider.create_workspace(invalid_dest)
    end

    # Test cleanup of non-existent workspace
    begin
      provider.cleanup_workspace('/non/existent/path')
      # Should not raise any exception
    rescue StandardError => e
      flunk "Cleanup should not raise exception for non-existent path: #{e.message}"
    end
  end
end
