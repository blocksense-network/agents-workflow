# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative 'test_helper'

class BinarySmokeTest < Minitest::Test
  include RepoTestHelper

  def test_all_binaries_start_and_get_task
    ALL_AGENT_TASK_BINARIES.product(ALL_GET_TASK_BINARIES).each do |ab, gb|
      repo, remote = setup_repo(:git)
      status, = run_agent_task(repo, branch: 'feat', lines: ['smoke'], push_to_remote: true, tool: ab)
      # agent-task should succeed with this binary
      assert_equal 0, status.exitstatus
      VCSRepo.new(repo).checkout_branch('feat')
      status2, output = run_get_task(repo, tool: gb)
      # get-task should retrieve the task description
      assert_equal 0, status2.exitstatus
      assert_includes output, 'smoke'
    ensure
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_start_work_all_binaries
    ALL_AGENT_TASK_BINARIES.product(ALL_START_WORK_BINARIES).each do |ab, sb|
      repo, remote = setup_repo(:git)
      status, = run_agent_task(repo, branch: 'feat', lines: ['work'], push_to_remote: true, tool: ab)
      # agent-task should succeed before start-work
      assert_equal 0, status.exitstatus
      VCSRepo.new(repo).checkout_branch('feat')
      status2, = run_start_work(repo, tool: sb)
      # start-work should configure the repo
      assert_equal 0, status2.exitstatus
    ensure
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end
end
