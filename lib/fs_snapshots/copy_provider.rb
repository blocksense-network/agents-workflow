# frozen_string_literal: true

require 'fileutils'
require_relative 'base_provider'

module FSSnapshots
  # Fallback provider that performs a regular copy of the workspace.
  # It attempts to use reflinks when supported for efficiency.
  class CopyProvider < BaseProvider
    def clone_workspace(dest)
      dest = File.expand_path(dest)
      FileUtils.mkdir_p(dest)
      # Use cp --reflink=auto if available for CoW copies, otherwise fall back
      cp_cmd = ['cp', '-a', '--reflink=auto', File.join(source, '.'), dest]
      system(*cp_cmd, out: File::NULL, err: File::NULL)
      dest
    end
  end
end
