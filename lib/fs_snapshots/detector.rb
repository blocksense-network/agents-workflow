# frozen_string_literal: true

require_relative 'copy_provider'

module FSSnapshots
  # Detects the best available snapshot provider for the current system.
  module Detector
    module_function

    # Determine which provider to use for the given workspace path.
    # Currently this only checks for presence of zfs or btrfs tools and
    # falls back to CopyProvider.
    def detect(_path)
      return provider_class('zfs') if command_available?('zfs')
      return provider_class('btrfs') if command_available?('btrfs')

      CopyProvider
    end

    def command_available?(cmd)
      system('which', cmd, out: File::NULL, err: File::NULL)
    end

    def provider_class(name)
      case name
      when 'zfs'
        begin
          require_relative 'zfs_provider'
          ZFSProvider
        rescue LoadError
          CopyProvider
        end
      when 'btrfs'
        begin
          require_relative 'btrfs_provider'
          BtrfsProvider
        rescue LoadError
          CopyProvider
        end
      else
        CopyProvider
      end
    end
  end
end
