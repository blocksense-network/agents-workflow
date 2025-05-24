# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative 'test_helper'

module StartTaskCases
  def assert_task_branch_created(repo, remote, branch)
    r = VCSRepo.new(repo)
    # agent-task should switch back to the main branch after creating the feature branch
    assert_equal r.default_branch, r.current_branch

    commit = r.tip_commit(branch)
    count = r.commit_count(r.default_branch, branch)
    # verify that exactly one commit was created on the new branch
    assert_equal 1, count

    files = r.files_in_commit(commit)
    # list the files from the new commit to ensure only the task file was added
    assert_equal 1, files.length
    assert_match(%r{\.agents/tasks/\d{4}/\d{2}/\d{2}-\d{4}-#{branch}}, files.first)

    remote_commit = case r.vcs_type
                    when :git
                      capture(remote, 'git', 'rev-parse', branch)
                    when :hg
                      capture(remote, 'hg', 'log', '-r', 'tip', '--template', '{node}')
                    when :fossil
                      sql = 'SELECT blob.uuid FROM tag JOIN tagxref ON tag.tagid=tagxref.tagid ' \
                            'JOIN blob ON tagxref.rid=blob.rid ' \
                            "WHERE tag.tagname='sym-#{branch}' " \
                            'ORDER BY tagxref.mtime DESC LIMIT 1'
                      capture(remote, 'fossil', 'sql', sql).gsub("'", '')
                    end
    # confirm the feature branch was pushed to the remote repository
    assert_equal commit, remote_commit
  end

  def test_clean_repo
    repo, remote = setup_repo(self.class::VCS_TYPE)
    status, = run_agent_task(repo, branch: 'feature', lines: ['task'], push_to_remote: true)
    assert_equal 0, status.exitstatus
    assert_task_branch_created(repo, remote, 'feature')
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_dirty_repo_staged
    repo, remote = setup_repo(self.class::VCS_TYPE)
    File.write(File.join(repo, 'foo.txt'), 'foo')
    r = VCSRepo.new(repo)
    r.add_file('foo.txt')
    status_before = r.working_copy_status
    status, = run_agent_task(repo, branch: 's1', lines: ['task'], push_to_remote: true)
    assert_equal 0, status.exitstatus
    # ensure staged changes are preserved and nothing else changed
    after = r.working_copy_status
    assert_equal status_before, after
    assert_task_branch_created(repo, remote, 's1')
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_dirty_repo_unstaged
    repo, remote = setup_repo(self.class::VCS_TYPE)
    File.write(File.join(repo, 'bar.txt'), 'bar')
    r = VCSRepo.new(repo)
    status_before = r.working_copy_status
    status, = run_agent_task(repo, branch: 's2', lines: ['task'], push_to_remote: true)
    assert_equal 0, status.exitstatus
    # unstaged modifications should remain exactly as they were
    after = r.working_copy_status
    assert_equal status_before, after
    assert_task_branch_created(repo, remote, 's2')
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_editor_failure
    repo, remote = setup_repo(self.class::VCS_TYPE)
    status, = run_agent_task(repo, branch: 'bad', lines: [], editor_exit: 1, push_to_remote: false)
    assert status.exitstatus != 0
    # when the editor fails, no branch should have been created
    refute VCSRepo.new(repo).branch_exists?('bad')
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_empty_file
    repo, remote = setup_repo(self.class::VCS_TYPE)
    status, = run_agent_task(repo, branch: 'empty', lines: [], push_to_remote: true)
    assert_equal 0, status.exitstatus
    r = VCSRepo.new(repo)
    branches = r.branches
    # an empty task file should still result in the new branch being created
    assert_includes branches, 'empty'
    expected_primary = r.vcs_type == :git ? "* #{r.default_branch}" : r.default_branch
    assert_includes branches, expected_primary
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_invalid_branch
    repo, remote = setup_repo(self.class::VCS_TYPE)
    status, _, executed = run_agent_task(repo, branch: 'inv@lid name', lines: ['task'], push_to_remote: false)
    refute executed, 'editor should not run when branch creation fails'
    assert status.exitstatus != 0
    # no branch should be created when the branch name is invalid
    refute VCSRepo.new(repo).branch_exists?('inv@lid name')
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end
end

class StartTaskGitTest < Minitest::Test
  include RepoTestHelper
  include StartTaskCases
  VCS_TYPE = :git
end

class StartTaskHgTest < Minitest::Test
  include RepoTestHelper
  include StartTaskCases
  VCS_TYPE = :hg
end

class StartTaskFossilTest < Minitest::Test
  include RepoTestHelper
  include StartTaskCases
  VCS_TYPE = :fossil
end
