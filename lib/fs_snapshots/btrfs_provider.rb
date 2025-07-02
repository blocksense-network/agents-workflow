# frozen_string_literal: true

require_relative 'base_provider'
require 'fileutils'

module FSSnapshots
  # Snapshot provider using Btrfs subvolume snapshots.
  class BtrfsProvider < BaseProvider
    def clone_workspace(dest)
      system('btrfs', 'subvolume', 'snapshot', source, dest, out: File::NULL, err: File::NULL)
      dest
    end
  end
end
