# frozen_string_literal: true

# Shared utilities for setting up filesystem environments in tests
module FilesystemSetupHelpers
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

  # Generate unique names for test resources
  def generate_unique_name(prefix = 'test')
    "#{prefix}_#{Process.pid}_#{Time.now.to_i}_#{rand(1000)}"
  end
end
