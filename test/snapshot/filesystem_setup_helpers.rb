# frozen_string_literal: true

# Shared utilities for setting up filesystem environments in tests
module FilesystemSetupHelpers
  # Common pattern for creating loop device image files
  def create_loop_device_image(image_path, size_mb, filesystem_type = nil)
    system('dd', 'if=/dev/zero', "of=#{image_path}", 'bs=1M', "count=#{size_mb}",
           out: File::NULL, err: File::NULL)

    case filesystem_type&.to_sym
    when :btrfs
      system('mkfs.btrfs', '-f', image_path, out: File::NULL, err: File::NULL)
    when :ext4
      system('mkfs.ext4', '-F', image_path, out: File::NULL, err: File::NULL)
    end
  end

  # Common pattern for mounting loop devices
  def mount_loop_device(image_path, mount_point, options = [])
    FileUtils.mkdir_p(mount_point)
    mount_options = ['loop'] + Array(options)
    system('mount', '-o', mount_options.join(','), image_path, mount_point,
           out: File::NULL, err: File::NULL)
  end

  # Common pattern for unmounting filesystems safely
  def safe_unmount(mount_point)
    return unless mount_point && File.exist?(mount_point)

    # Try multiple times as unmount can sometimes be delayed
    3.times do
      break if system('umount', mount_point, out: File::NULL, err: File::NULL)

      sleep(0.1)
    end
  end

  # Common pattern for initializing test repository content
  def initialize_test_repo(repo_dir, content = {})
    default_content = {
      'README.md' => 'test repo content',
      'test_file.txt' => 'additional content'
    }

    content = default_content.merge(content)

    content.each do |filename, file_content|
      File.write(File.join(repo_dir, filename), file_content)
    end
  end

  # Check if we have the necessary privileges for filesystem operations
  def check_filesystem_privileges(operation_type = :mount)
    case operation_type
    when :mount
      # Test if we can perform mount operations
      test_dir = Dir.mktmpdir('privilege_test')
      test_file = File.join(test_dir, 'test.img')

      begin
        create_loop_device_image(test_file, 1)
        mount_result = mount_loop_device(test_file, File.join(test_dir, 'mount'))
        safe_unmount(File.join(test_dir, 'mount')) if mount_result
        mount_result
      rescue StandardError
        false
      ensure
        FileUtils.remove_entry(test_dir) if test_dir && File.exist?(test_dir)
      end
    else
      true # Assume other operations are allowed
    end
  end

  # Generate unique names for test resources
  def generate_unique_name(prefix = 'test')
    "#{prefix}_#{Process.pid}_#{Time.now.to_i}_#{rand(1000)}"
  end
end
