# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require_relative 'test_helper'
require_relative '../lib/fs_snapshots'

# Tests for snapshot provider detection logic
class TestSnapshotDetector < Minitest::Test
  def setup
    @tmp_path = Dir.mktmpdir('workspace')
  end

  def teardown
    FileUtils.remove_entry(@tmp_path) if @tmp_path && File.exist?(@tmp_path)
  end

  def test_detects_copy_provider_when_no_tools
    klass = FSSnapshots::Detector.detect(@tmp_path)
    assert_equal FSSnapshots::CopyProvider, klass
  end

  def test_detects_zfs_provider_when_zfs_command_present
    Dir.mktmpdir('fakebin') do |dir|
      File.write(File.join(dir, 'zfs'), "#!/bin/sh\nexit 0\n")
      File.chmod(0o755, File.join(dir, 'zfs'))
      original = ENV.fetch('PATH', nil)
      ENV['PATH'] = "#{dir}:#{original}"
      begin
        klass = FSSnapshots::Detector.detect(@tmp_path)
        assert_equal FSSnapshots::ZFSProvider, klass
      ensure
        ENV['PATH'] = original
      end
    end
  end
end
