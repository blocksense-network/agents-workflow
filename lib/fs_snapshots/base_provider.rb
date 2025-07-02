# frozen_string_literal: true

module FSSnapshots
  # Base class for snapshot providers.
  # Subclasses must implement the `clone_workspace` method which clones the
  # source directory into the target directory using the provider's
  # snapshotting mechanism.
  class BaseProvider
    attr_reader :source

    def initialize(source)
      @source = File.expand_path(source)
    end

    # Clone the source directory into the given target directory.
    # @param dest [String] directory where the snapshot should be created
    # @return [String] path to the created snapshot
    def clone_workspace(dest)
      raise NotImplementedError, 'clone_workspace must be implemented by subclasses'
    end
  end
end
