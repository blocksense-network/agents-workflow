# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require_relative 'test_helper'

class SetupCommandTest < Minitest::Test
  include RepoTestHelper

  def test_setup_prints_versions
    RepoTestHelper::ALL_AGENT_TASK_BINARIES.each do |bin|
      Dir.mktmpdir('work') do |dir|
        status, output = run_agent_task_setup(dir, tool: bin)
        # setup command should succeed
        assert_equal 0, status.exitstatus
        # output should mention codex and goose versions
        assert_match(/codex:/, output)
        assert_match(/goose:/, output)
      end
    end
  end
end
