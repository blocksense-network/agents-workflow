require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'

ROOT = File.expand_path('..', __dir__)
START_TASK = File.join(ROOT, 'bin', 'start-task')

# helper to run git commands in repo
def git(repo, *args)
  cmd = ['git', *args]
  system({'GIT_CONFIG_NOSYSTEM'=>'1'}, *cmd, chdir: repo, out: File::NULL, err: File::NULL)
end

def setup_git_repo
  remote = Dir.mktmpdir('remote')
  system('git', 'init', '--bare', remote, out: File::NULL)
  repo = Dir.mktmpdir('repo')
  system('git', 'init', '-b', 'main', repo, out: File::NULL)
  git(repo, 'config', 'user.email', 'tester@example.com')
  git(repo, 'config', 'user.name', 'Tester')
  File.write(File.join(repo, 'README.md'), 'initial')
  git(repo, 'add', 'README.md')
  git(repo, 'commit', '-m', 'initial')
  git(repo, 'remote', 'add', 'origin', remote)
  [repo, remote]
end

# run start-task with given editor content
# branch: branch name to pass as argument
# lines: array of lines to write to temp file
# editor_exit: exit code for editor
# input: text to send to start-task via stdin

def run_start_task(repo, branch:, lines: [], editor_exit: 0, input: "y\n")
  dir = Dir.mktmpdir('editor')
  script = File.join(dir, 'fake_editor.sh')
  marker = File.join(dir, 'called')
  File.write(script, <<~SH)
    #!/bin/sh
    echo yes > #{marker}
    cat <<'EOS' > "$1"
    #{lines.join("\n")}
    EOS
    exit #{editor_exit}
  SH
  File.chmod(0755, script)
  output = nil
  status = nil
  Dir.chdir(repo) do
    IO.popen({'EDITOR'=>script}, [START_TASK, branch], 'r+') do |io|
      io.write input
      io.close_write
      output = io.read
    end
    status = $?
  end
  executed = File.exist?(marker)
  FileUtils.remove_entry(dir)
  [status, output, executed]
end

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
