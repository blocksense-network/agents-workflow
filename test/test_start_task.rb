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

def git_capture(repo, *args)
  cmd = ['git', *args]
  `#{cmd.shelljoin}`
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
# lines is array of lines to write to temp file
# editor_exit: exit code for editor
# input: text to send to start-task via stdin

def run_start_task(repo, lines: [], editor_exit: 0, input: "y\n")
  dir = Dir.mktmpdir('editor')
  script = File.join(dir, 'fake_editor.sh')
  File.write(script, <<~SH)
    #!/bin/sh
    cat <<'EOS' > "$1"
    #{lines.join("\n")}
    EOS
    exit #{editor_exit}
  SH
  File.chmod(0755, script)
  output = nil
  status = nil
  Dir.chdir(repo) do
    IO.popen({'EDITOR'=>script}, [START_TASK], 'r+') do |io|
      io.write input
      io.close_write
      output = io.read
    end
    status = $?
  end
  FileUtils.remove_entry(dir)
  [status, output]
end

class StartTaskGitTest < Minitest::Test
  def test_clean_repo
    repo, remote = setup_git_repo
    status, _ = run_start_task(repo, lines: ['branch: feature', 'task'])
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
    status, _ = run_start_task(repo, lines: ['branch: s1', 'task'])
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
    status, _ = run_start_task(repo, lines: ['branch: s2', 'task'])
    assert_equal 0, status.exitstatus
    # unstaged modifications should remain exactly as they were
    assert_equal status_before, `git -C #{repo} status --porcelain`
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end

  def test_editor_failure
    repo, remote = setup_git_repo
    status, _ = run_start_task(repo, lines: ['branch: bad'], editor_exit: 1)
    assert status.exitstatus != 0
    # when the editor fails, no branch should have been created
    refute `git -C #{repo} branch --list bad`.strip.size > 0
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end

  def test_empty_file
    repo, remote = setup_git_repo
    status, _ = run_start_task(repo, lines: [])
    assert_equal 0, status.exitstatus
    branches = `git -C #{repo} branch --list`.split("\n").map(&:strip)
    # saving an empty task file should leave only the main branch
    assert_equal ['* main'], branches
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end

  def test_branch_sanitization
    repo, remote = setup_git_repo
    status, _ = run_start_task(repo, lines: ['branch: inv@lid name', 'task'])
    assert_equal 0, status.exitstatus
    # the branch name should be sanitized of invalid characters
    assert `git -C #{repo} branch --list inv-lid-name`.strip.size > 0
  ensure
    FileUtils.remove_entry(repo)
    FileUtils.remove_entry(remote)
  end
end