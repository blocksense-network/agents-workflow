require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative 'test_helper'

include RepoTestHelper


class StartTaskGitTest < Minitest::Test
  def test_clean_repo
    repo, remote = setup_git_repo
    status, _, _ = run_start_task(repo, branch: 'feature', lines: ['task'])
    assert_equal 0, status.exitstatus
    # start-task should switch back to main after creating the feature branch
    assert_equal 'main', `git -C #{repo} rev-parse --abbrev-ref HEAD`.strip
    # verify that exactly one commit was created on the new branch
    assert_equal 1, `git -C #{repo} rev-list main..feature --count`.to_i
    commit=`git -C #{repo} rev-parse feature`
    # list the files from the new commit to ensure only the task file was added
    files=`git -C #{repo} diff-tree --no-commit-id --name-only -r #{commit}`.split
    assert_equal 1, files.length
    assert_match %r{\.agents/tasks/\d{4}/\d{2}/\d{2}-\d{4}-feature}, files.first
    # confirm the feature branch was pushed to the remote repository
    assert_equal commit.strip, `git --git-dir=#{remote} rev-parse feature`.strip
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end

  def test_dirty_repo_staged
    repo, remote = setup_git_repo
    File.write(File.join(repo, 'foo.txt'), 'foo')
    git(repo, 'add', 'foo.txt')
    status, _, _ = run_start_task(repo, branch: 's1', lines: ['task'])
    assert_equal 0, status.exitstatus
    # ensure staged changes are restored and nothing else changed
    assert_equal '', `git -C #{repo} status --porcelain`
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end

  def test_dirty_repo_unstaged
    repo, remote = setup_git_repo
    File.write(File.join(repo, 'bar.txt'), 'bar')
    status_before = `git -C #{repo} status --porcelain`
    status, _, _ = run_start_task(repo, branch: 's2', lines: ['task'])
    assert_equal 0, status.exitstatus
    # unstaged modifications should remain exactly as they were
    assert_equal status_before, `git -C #{repo} status --porcelain`
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end

  def test_editor_failure
    repo, remote = setup_git_repo
    status, _, _ = run_start_task(repo, branch: 'bad', lines: [], editor_exit: 1)
    assert status.exitstatus != 0
    # when the editor fails, no branch should have been created
    refute `git -C #{repo} branch --list bad`.strip.size > 0
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end

  def test_empty_file
    repo, remote = setup_git_repo
    status, _, _ = run_start_task(repo, branch: 'empty', lines: [])
    assert_equal 0, status.exitstatus
    branches = `git -C #{repo} branch --list`.split("\n").map(&:strip)
    # an empty task file should still result in the new branch being created
    assert_includes branches, 'empty'
    assert_includes branches, '* main'
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end

  def test_invalid_branch
    repo, remote = setup_git_repo
    status, _, executed = run_start_task(repo, branch: 'inv@lid name', lines: ['task'])
    refute executed, 'editor should not run when branch creation fails'
    assert status.exitstatus != 0
    # no branch should be created when the branch name is invalid
    refute `git -C #{repo} branch --list 'inv@lid name'`.strip.size > 0
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end
end
