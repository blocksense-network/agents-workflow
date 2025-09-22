# frozen_string_literal: true

# Shared behavior for providers that support quota/space limit testing
module ProviderQuotaTestBehavior
  # Test quota enforcement and space limits
  def test_provider_quota_limits
    return skip "Quota testing not supported for #{provider_class_name}" unless supports_quota_testing?

    setup_quota_environment
    provider = create_test_provider
    workspace_dir = create_workspace_destination('quota_test')

    begin
      # Create workspace
      result_path = provider.create_workspace(workspace_dir)

      # Try to exceed quota
      large_file = File.join(result_path, 'large_file.dat')
      quota_exceeded = false

      begin
        File.write(large_file, 'x' * quota_test_size)
      rescue Errno::ENOSPC, Errno::EDQUOT
        quota_exceeded = true
      end

      # Verify quota behavior (implementation-specific)
      verify_quota_behavior(quota_exceeded)
    ensure
      provider.cleanup_workspace(workspace_dir) if File.exist?(workspace_dir)
      cleanup_quota_environment
    end
  end

  private

  # Abstract methods for quota testing
  def supports_quota_testing?
    false
  end

  def setup_quota_environment
    # Override in subclasses that support quota testing
  end

  def cleanup_quota_environment
    # Override in subclasses that support quota testing
  end

  def quota_test_size
    15 * 1024 * 1024 # 15MB default
  end

  def verify_quota_behavior(quota_exceeded)
    # Override in subclasses for provider-specific quota verification
  end

  def provider_class_name
    self.class.name.gsub(/^Test|Test$/, '')
  end
end
