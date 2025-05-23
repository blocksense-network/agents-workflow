# frozen_string_literal: true

require 'English'
require 'rbconfig'
require 'tmpdir'
require_relative '../bin/lib/vcs_repo'

module RepoTestHelper
  ROOT = File.expand_path('..', __dir__)
  AGENT_TASK = File.join(ROOT, 'bin', 'agent-task')
  GET_TASK = File.join(ROOT, 'bin', 'get-task')

  def windows?
    RbConfig::CONFIG['host_os'] =~ /mswin|mingw|cygwin/
  end

  def git(repo, *args)
    cmd = ['git', *args]
    system({ 'GIT_CONFIG_NOSYSTEM' => '1' }, *cmd, chdir: repo, out: File::NULL, err: File::NULL)
  end

  def hg(repo, *args)
    cmd = ['hg', *args]
    system(*cmd, chdir: repo, out: File::NULL, err: File::NULL)
  end

  def capture(repo, tool, *args)
    IO.popen([tool, *args], chdir: repo, &:read).strip
  end

  def setup_repo(vcs_type)
    remote = Dir.mktmpdir('remote')
    repo = Dir.mktmpdir('repo')

    case vcs_type
    when :git
      system('git', 'init', '--bare', remote, out: File::NULL)
      system('git', 'init', '-b', 'main', repo, out: File::NULL)
      git(repo, 'config', 'user.email', 'tester@example.com')
      git(repo, 'config', 'user.name', 'Tester')
      File.write(File.join(repo, 'README.md'), 'initial')
      git(repo, 'add', 'README.md')
      git(repo, 'commit', '-m', 'initial')
      git(repo, 'remote', 'add', 'origin', remote)
    when :hg
      system('hg', 'init', remote, out: File::NULL)
      system('hg', 'init', repo, out: File::NULL)
      File.write(File.join(repo, 'README.md'), 'initial')
      Dir.chdir(repo) do
        hg(repo, 'add', 'README.md')
        hg(repo, 'commit', '-m', 'initial', '-u', 'Tester <tester@example.com>')
        hgrc = File.join('.hg', 'hgrc')
        File.open(hgrc, 'a') do |f|
          f.puts '[paths]'
          f.puts "default = #{remote}"
        end
      end
    else
      raise "Unsupported VCS type '#{vcs_type}'"
    end

    [repo, remote]
  end

  def run_agent_task(repo, branch:, lines: [], editor_exit: 0, input: "y\n")
    dir = Dir.mktmpdir('editor')
    script = File.join(dir, 'fake_editor.rb')
    marker = File.join(dir, 'called')
    File.write(script, <<~RB)
      #!/usr/bin/env ruby
      File.write('#{marker}', "yes\n")
      File.open(ARGV[0], 'w') do |f|
        content = #{lines.inspect}.join("\n")
        f.write(content)
        f.write("\n") unless content.empty?
      end
      exit #{editor_exit}
    RB
    File.chmod(0o755, script)
    output = nil
    status = nil
    Dir.chdir(repo) do
      cmd = windows? ? ['ruby', AGENT_TASK, branch] : [AGENT_TASK, branch]
      editor_cmd = windows? ? "ruby #{script}" : script
      IO.popen({ 'EDITOR' => editor_cmd }, cmd, 'r+') do |io|
        io.write input
        io.close_write
        output = io.read
      end
      status = $CHILD_STATUS
    end
    executed = File.exist?(marker)
    FileUtils.remove_entry(dir)
    [status, output, executed]
  end

  def run_get_task(repo)
    output = nil
    status = nil
    Dir.chdir(repo) do
      cmd = windows? ? ['ruby', GET_TASK] : [GET_TASK]
      output = IO.popen(cmd, &:read)
      status = $CHILD_STATUS
    end
    [status, output]
  end
end
