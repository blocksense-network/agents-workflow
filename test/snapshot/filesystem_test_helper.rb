# frozen_string_literal: true

require 'tmpdir'
require 'fileutils'

# Shared helper methods for filesystem testing using loop devices
module FilesystemTestHelper
  # Creates a loop-mounted filesystem for testing purposes
  # @param fs_type [String] The filesystem type ('ext4', 'xfs', 'btrfs')
  # @param name [String] A unique name for this filesystem instance
  # @param size_mb [Integer] Size of the filesystem in megabytes
  # @param test_dir [String] Base directory for creating filesystem images
  # @return [String] The mount point path
  # @raise [RuntimeError] If filesystem creation or mounting fails
  def create_loop_filesystem(fs_type, name, size_mb: 100, test_dir: @test_dir)
    image_file = File.join(test_dir, "#{name}.img")
    mount_point = File.join(test_dir, "#{name}_mount")

    # Create image file using dd
    unless system('dd', 'if=/dev/zero', "of=#{image_file}", 'bs=1M', "count=#{size_mb}",
                  out: File::NULL, err: File::NULL)
      raise "Failed to create filesystem image: #{image_file}"
    end

    # Create filesystem based on type
    case fs_type
    when 'ext4'
      unless system('mkfs.ext4', '-F', image_file, out: File::NULL, err: File::NULL)
        raise "Failed to create ext4 filesystem on #{image_file}"
      end
    when 'xfs'
      unless system('mkfs.xfs', '-f', image_file, out: File::NULL, err: File::NULL)
        raise "Failed to create xfs filesystem on #{image_file}"
      end
    when 'btrfs'
      unless system('mkfs.btrfs', '-f', image_file, out: File::NULL, err: File::NULL)
        raise "Failed to create btrfs filesystem on #{image_file}"
      end
    else
      raise "Unsupported filesystem type: #{fs_type}"
    end

    # Mount filesystem
    FileUtils.mkdir_p(mount_point)
    unless system('mount', '-o', 'loop', image_file, mount_point, out: File::NULL, err: File::NULL)
      raise "Failed to mount #{fs_type} filesystem - may need root privileges"
    end

    # Track filesystem for cleanup (assumes @filesystems and @mount_points exist)
    if respond_to?(:track_filesystem)
      track_filesystem(image_file, mount_point, fs_type)
    elsif instance_variable_defined?(:@filesystems) && instance_variable_defined?(:@mount_points)
      @filesystems << { image: image_file, mount: mount_point, type: fs_type }
      @mount_points << mount_point
    end

    mount_point
  end

  # Cleanup all tracked filesystems by unmounting them
  def cleanup_all_filesystems
    return unless instance_variable_defined?(:@mount_points)

    @mount_points.each do |mount_point|
      system('umount', mount_point, out: File::NULL, err: File::NULL) if File.exist?(mount_point)
    end
    @filesystems&.clear
    @mount_points.clear
  end

  # Get filesystem usage in bytes using df
  # @param mount_point [String] The filesystem mount point
  # @return [Integer] Used space in bytes
  def get_filesystem_used_space(mount_point)
    output = `df -B1 #{mount_point} 2>/dev/null | tail -1`
    return 0 if output.empty?

    fields = output.split
    return 0 if fields.length < 3

    fields[2].to_i # Used space in bytes
  end
end
