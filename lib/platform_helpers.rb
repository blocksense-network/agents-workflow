# frozen_string_literal: true

# Utility helpers for detecting the host platform.
# These are mixed into Kernel so they can be used anywhere.

module PlatformHelpers
  module_function

  def windows?
    RbConfig::CONFIG['host_os'] =~ /mswin|mingw|cygwin/
  end

  def linux?
    RbConfig::CONFIG['host_os'] =~ /linux/
  end

  def macos?
    RbConfig::CONFIG['host_os'] =~ /darwin|mac os/
  end
end

# Provide global helper methods for convenience
module Kernel
  include PlatformHelpers
end
