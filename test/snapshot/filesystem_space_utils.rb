# frozen_string_literal: true

# Shared utilities for measuring filesystem space usage across different filesystem types
module FilesystemSpaceUtils
  # Parse size string with units (B, K, KB, KiB, M, MB, MiB, etc.) to bytes
  def parse_size_to_bytes(size_string)
    return 0 if size_string.nil? || size_string.empty?

    match = size_string.match(/(\d+(?:\.\d+)?)\s*(\w*)/)
    return 0 unless match

    value = match[1].to_f
    unit = match[2].upcase

    case unit
    when '', 'B', 'BYTES'
      value.to_i
    when 'K', 'KB', 'KIB'
      (value * 1024).to_i
    when 'M', 'MB', 'MIB'
      (value * 1024 * 1024).to_i
    when 'G', 'GB', 'GIB'
      (value * 1024 * 1024 * 1024).to_i
    when 'T', 'TB', 'TIB'
      (value * 1024 * 1024 * 1024 * 1024).to_i
    else
      value.to_i
    end
  end

  # Get Btrfs filesystem usage in bytes
  def btrfs_filesystem_used_space(mount_point)
    output = `btrfs filesystem usage #{mount_point} 2>/dev/null`
    match = output.match(/Used:\s+(\d+(?:\.\d+)?)\s*(\w+)/)
    return 0 unless match

    parse_size_to_bytes("#{match[1]}#{match[2]}")
  end

  # Get ZFS pool usage in bytes
  def zfs_pool_used_space(pool_name)
    output = `zpool list -H -o used #{pool_name} 2>/dev/null`.strip
    return 0 if output.empty?

    parse_size_to_bytes(output)
  end

  # Get general filesystem usage using df
  def df_filesystem_used_space(mount_point)
    output = `df -B1 #{mount_point} 2>/dev/null | tail -1`.strip
    return 0 if output.empty?

    # df output: Filesystem 1B-blocks Used Available Use% Mounted
    fields = output.split
    return 0 if fields.length < 3

    fields[2].to_i # Used space in bytes
  end

  # Generic method that tries to determine the best space measurement approach
  def measure_filesystem_space(path_or_pool, filesystem_type = nil)
    case filesystem_type&.to_sym
    when :btrfs
      btrfs_filesystem_used_space(path_or_pool)
    when :zfs
      zfs_pool_used_space(path_or_pool)
    else
      # Try to auto-detect or fall back to df
      if File.directory?(path_or_pool)
        df_filesystem_used_space(path_or_pool)
      else
        0 # Unknown
      end
    end
  end
end
