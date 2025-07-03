# frozen_string_literal: true

require_relative 'provider_core_test_behavior'
require_relative 'provider_performance_test_behavior'

# Shared test behaviors for all snapshot providers
# This module combines all provider test behaviors into a single interface
module ProviderSharedBehavior
  include ProviderCoreTestBehavior
  include ProviderPerformanceTestBehavior
end
