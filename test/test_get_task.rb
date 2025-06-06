# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require 'open3'
require_relative 'test_helper'

module GetTaskCases
  def test_get_task_after_start
    RepoTestHelper::AGENT_TASK_BINARIES.product(RepoTestHelper::GET_TASK_BINARIES).each do |ab, gb|
      repo, remote = setup_repo(self.class::VCS_TYPE)
      status, = run_agent_task(repo, branch: 'feat', lines: ['my task'], push_to_remote: true, tool: ab)
      # agent-task should succeed
      assert_equal 0, status.exitstatus
      VCSRepo.new(repo).checkout_branch('feat')
      status2, output = run_get_task(repo, tool: gb)
      # get-task should print the task description
      assert_equal 0, status2.exitstatus
      assert_includes output, 'my task'
    ensure
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_get_task_on_work_branch
    RepoTestHelper::AGENT_TASK_BINARIES.product(RepoTestHelper::GET_TASK_BINARIES).each do |ab, gb|
      repo, remote = setup_repo(self.class::VCS_TYPE)
      status, = run_agent_task(repo, branch: 'feat', lines: ['follow task'], push_to_remote: true, tool: ab)
      # agent-task should succeed
      assert_equal 0, status.exitstatus
      r = VCSRepo.new(repo)
      r.checkout_branch('feat')
      r.create_local_branch('work')
      status2, output = run_get_task(repo, tool: gb)
      # even after switching to a different work branch the task should be retrievable
      assert_equal 0, status2.exitstatus
      assert_includes output, 'follow task'
    ensure
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end
end

class GetTaskGitTest < Minitest::Test
  include RepoTestHelper
  include GetTaskCases
  VCS_TYPE = :git
end

# These tests are temporarily disabled until we get git to work
# class GetTaskHgTest < Minitest::Test
#   include RepoTestHelper
#   include GetTaskCases
#   VCS_TYPE = :hg
# end
#
# class GetTaskFossilTest < Minitest::Test
#   include RepoTestHelper
#   include GetTaskCases
#   VCS_TYPE = :fossil
# end
