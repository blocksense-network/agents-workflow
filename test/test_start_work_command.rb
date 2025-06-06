# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative 'test_helper'

module StartWorkCases
  def assert_work_setup(repo, remote_url)
    # remote should be added correctly
    assert_equal remote_url, capture(repo, 'git', 'remote', 'get-url', 'target_remote')
    # user.name should match the last commit author
    assert_equal 'Tester', capture(repo, 'git', 'config', '--get', 'user.name')
    # user.email should also match
    assert_equal 'tester@example.com', capture(repo, 'git', 'config', '--get', 'user.email')
  end

  def test_start_work_after_start
    RepoTestHelper::AGENT_TASK_BINARIES.product(RepoTestHelper::START_WORK_BINARIES).each do |ab, sb|
      repo, remote = setup_repo(self.class::VCS_TYPE)
      status, = run_agent_task(repo, branch: 'feat', lines: ['task'], push_to_remote: true, tool: ab)
      # agent-task should succeed
      assert_equal 0, status.exitstatus
      VCSRepo.new(repo).checkout_branch('feat')
      status2, = run_start_work(repo, tool: sb)
      # start-work should succeed
      assert_equal 0, status2.exitstatus
      assert_work_setup(repo, remote)
    ensure
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_start_work_on_work_branch
    RepoTestHelper::AGENT_TASK_BINARIES.product(RepoTestHelper::START_WORK_BINARIES).each do |ab, sb|
      repo, remote = setup_repo(self.class::VCS_TYPE)
      status, = run_agent_task(repo, branch: 'feat', lines: ['other'], push_to_remote: true, tool: ab)
      # agent-task should succeed
      assert_equal 0, status.exitstatus
      r = VCSRepo.new(repo)
      r.checkout_branch('feat')
      r.create_local_branch('work')
      status2, = run_start_work(repo, tool: sb)
      # start-work should succeed on a different branch
      assert_equal 0, status2.exitstatus
      assert_work_setup(repo, remote)
    ensure
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_start_work_from_parent_directory
    RepoTestHelper::AGENT_TASK_BINARIES.product(RepoTestHelper::START_WORK_BINARIES).each do |ab, sb|
      repo, remote = setup_repo(self.class::VCS_TYPE)
      status, = run_agent_task(repo, branch: 'feat', lines: ['outer'], push_to_remote: true, tool: ab)
      # agent-task should succeed
      assert_equal 0, status.exitstatus
      outer = Dir.mktmpdir('outer')
      FileUtils.mv(repo, File.join(outer, 'repo'))
      status2, = run_start_work(outer, tool: sb)
      # start-work should succeed when launched from parent directory
      assert_equal 0, status2.exitstatus
      assert_work_setup(File.join(outer, 'repo'), remote)
    ensure
      FileUtils.remove_entry(outer) if outer && File.exist?(outer)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_start_work_from_parent_directory_multiple_repos
    RepoTestHelper::AGENT_TASK_BINARIES.product(RepoTestHelper::START_WORK_BINARIES).each do |ab, sb|
      repo_a, remote_a = setup_repo(self.class::VCS_TYPE)
      status, = run_agent_task(repo_a, branch: 'feat', lines: ['a'], push_to_remote: true, tool: ab)
      # first repo setup
      assert_equal 0, status.exitstatus
      repo_b, remote_b = setup_repo(self.class::VCS_TYPE)
      status, = run_agent_task(repo_b, branch: 'feat', lines: ['b'], push_to_remote: true, tool: ab)
      # second repo setup
      assert_equal 0, status.exitstatus
      outer = Dir.mktmpdir('outer')
      FileUtils.mv(repo_a, File.join(outer, 'a'))
      FileUtils.mv(repo_b, File.join(outer, 'b'))
      status2, = run_start_work(outer, tool: sb)
      # start-work should configure all repositories
      assert_equal 0, status2.exitstatus
      assert_work_setup(File.join(outer, 'a'), remote_a)
      assert_work_setup(File.join(outer, 'b'), remote_b)
    ensure
      FileUtils.remove_entry(outer) if outer && File.exist?(outer)
      FileUtils.remove_entry(remote_a) if remote_a && File.exist?(remote_a)
      FileUtils.remove_entry(remote_b) if remote_b && File.exist?(remote_b)
    end
  end
end

class StartWorkGitTest < Minitest::Test
  include RepoTestHelper
  include StartWorkCases
  VCS_TYPE = :git
end
