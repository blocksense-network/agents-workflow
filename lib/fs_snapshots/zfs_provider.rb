# frozen_string_literal: true

require_relative 'base_provider'
require 'fileutils'

module FSSnapshots
  # Snapshot provider using ZFS snapshots and clones.
  class ZFSProvider < BaseProvider
    SNAP_PREFIX = 'agents_workflow'

    def clone_workspace(dest)
      timestamp = Time.now.utc.strftime('%Y%m%d%H%M%S')
      snapshot = "#{source}@#{SNAP_PREFIX}_#{timestamp}"
      system('zfs', 'snapshot', snapshot, out: File::NULL, err: File::NULL)
      system('zfs', 'clone', snapshot, dest, out: File::NULL, err: File::NULL)
      dest
    end
  end
end
