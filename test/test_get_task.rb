# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require 'open3'
require_relative 'test_helper'

class GetTaskTest < Minitest::Test
  include RepoTestHelper
  def test_get_task_after_start
    repo, remote = setup_git_repo
    status, = run_agent_task(repo, branch: 'feat', lines: ['my task'])
    # agent-task should succeed
    assert_equal 0, status.exitstatus
    git(repo, 'checkout', 'feat')
    status2, output = run_get_task(repo)
    # get-task should print the task description
    assert_equal 0, status2.exitstatus
    assert_includes output, 'my task'
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end

  def test_get_task_on_work_branch
    repo, remote = setup_git_repo
    status, = run_agent_task(repo, branch: 'feat', lines: ['follow task'])
    # agent-task should succeed
    assert_equal 0, status.exitstatus
    git(repo, 'checkout', 'feat')
    git(repo, 'checkout', '-b', 'work')
    status2, output = run_get_task(repo)
    # even after switching to a different work branch the task should be retrievable
    assert_equal 0, status2.exitstatus
    assert_includes output, 'follow task'
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end
end
