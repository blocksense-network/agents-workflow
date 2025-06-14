# frozen_string_literal: true

require 'English'
require 'rbconfig'
require 'tmpdir'
require_relative '../lib/vcs_repo'

# Add debugging support when ENV variable is set
if ENV['RUBY_DEBUG'] || ENV['DEBUG_TESTS']
  require_relative '../lib/pry_debug'
  puts '🔍 Debug mode enabled for tests'
end

module RepoTestHelper # rubocop:disable Metrics/ModuleLength
  ROOT = File.expand_path('..', __dir__)
  AGENT_TASK = File.join(ROOT, 'bin', 'agent-task')
  GET_TASK = File.join(ROOT, 'bin', 'get-task')

  GEM_HOME = Dir.mktmpdir('gem_home')
  Gem.paths = { 'GEM_HOME' => GEM_HOME, 'GEM_PATH' => GEM_HOME }
  Dir.chdir(ROOT) do
    system('gem', 'build', 'agent-task.gemspec', out: File::NULL)
    gem_file = Dir['agent-task-*.gem'].first
    system('gem', 'install', '--no-document', '--install-dir', GEM_HOME, gem_file,
           out: File::NULL)
    FileUtils.rm_f(gem_file)
  end
  AGENT_TASK_GEM = File.join(GEM_HOME, 'bin', 'agent-task')
  GET_TASK_GEM = File.join(GEM_HOME, 'bin', 'get-task')
  GEM_AGENT_TASK_SCRIPT = File.join(ROOT, 'scripts', 'gem_agent_task.rb')
  GEM_GET_TASK_SCRIPT = File.join(ROOT, 'scripts', 'gem_get_task.rb')
  GEM_START_WORK_SCRIPT = File.join(ROOT, 'scripts', 'gem_start_work.rb')
  GEM_ENV = {
    'GEM_HOME' => GEM_HOME,
    'GEM_PATH' => GEM_HOME,
    # Prevent Git from prompting for authentication in tests
    'GIT_CONFIG_NOSYSTEM' => '1',
    'GIT_TERMINAL_PROMPT' => '0',
    'GIT_ASKPASS' => 'echo',
    'SSH_ASKPASS' => 'echo'
  }.freeze

  AGENT_TASK_BINARIES = [AGENT_TASK].freeze
  GET_TASK_BINARIES = [GET_TASK].freeze
  START_WORK = File.join(ROOT, 'bin', 'start-work')
  START_WORK_GEM = File.join(GEM_HOME, 'bin', 'start-work')
  START_WORK_BINARIES = [START_WORK].freeze

  ALL_AGENT_TASK_BINARIES = [AGENT_TASK, AGENT_TASK_GEM, GEM_AGENT_TASK_SCRIPT].freeze
  ALL_GET_TASK_BINARIES = [GET_TASK, GET_TASK_GEM, GEM_GET_TASK_SCRIPT].freeze
  ALL_START_WORK_BINARIES = [START_WORK, START_WORK_GEM, GEM_START_WORK_SCRIPT].freeze

  def windows?
    RbConfig::CONFIG['host_os'] =~ /mswin|mingw|cygwin/
  end

  def git(repo, *args)
    cmd = ['git', *args]
    env = {
      'GIT_CONFIG_NOSYSTEM' => '1',
      'GIT_TERMINAL_PROMPT' => '0',
      'GIT_ASKPASS' => 'echo',
      'SSH_ASKPASS' => 'echo'
    }
    system(env, *cmd, chdir: repo, out: File::NULL, err: File::NULL)
  end

  def hg(repo, *args)
    cmd = ['hg', *args]
    system(*cmd, chdir: repo, out: File::NULL, err: File::NULL)
  end

  def fossil(repo, *args)
    cmd = ['fossil', *args]
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
          f.puts '[ui]'
          f.puts 'username = Tester <tester@example.com>'
          f.puts '[paths]'
          f.puts "default = #{remote}"
        end
      end
    when :fossil
      remote_file = File.join(remote, 'remote.fossil')
      system('fossil', 'init', '--admin-user', 'Tester', remote_file, out: File::NULL)
      Dir.chdir(repo) do
        system('fossil', 'open', remote_file, '--user', 'Tester', out: File::NULL)
        fossil(repo, 'user', 'default', 'Tester')
        fossil(repo, 'remote', "file://#{remote_file}")
        fossil(repo, 'settings', 'autosync', 'off')
        File.write('README.md', 'initial')
        fossil(repo, 'add', 'README.md')
        fossil(repo, 'commit', '-m', 'initial', '--user', 'Tester')
      end
      remote = repo
    else
      raise "Unsupported VCS type '#{vcs_type}'"
    end

    [repo, remote]
  end

  # push_to_remote option avoids interactive prompts in CI
  # rubocop:disable Metrics/ParameterLists
  def run_agent_task(repo, branch:, lines: [], editor_exit: 0, input: nil, push_to_remote: nil, prompt: nil,
                     prompt_file: nil, devshell: nil, tool: AGENT_TASK)
    dir = nil
    script = nil
    marker = nil

    unless prompt || prompt_file
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
    end
    output = nil
    status = nil
    Dir.chdir(repo) do
      cmd = windows? ? ['ruby', tool] : [tool]
      cmd << branch if branch
      cmd << "--push-to-remote=#{push_to_remote}" unless push_to_remote.nil?
      cmd << "--prompt=#{prompt}" if prompt
      cmd << "--prompt-file=#{prompt_file}" if prompt_file
      cmd << "--devshell=#{devshell}" if devshell
      env = GEM_ENV.dup
      if script
        env['EDITOR'] = windows? ? "ruby #{script}" : script
      end
      answer = input
      if answer.nil?
        repo_type = VCSRepo.new(repo).vcs_type
        answer = repo_type == :fossil ? "n\n" : "y\n"
      end
      IO.popen(env, cmd, 'r+') do |io|
        io.write(answer) if push_to_remote.nil?
        io.close_write
        output = io.read
      end
      status = $CHILD_STATUS
    end
    executed = marker && File.exist?(marker)
    FileUtils.remove_entry(dir) if dir
    [status, output, executed]
  end
  # rubocop:enable Metrics/ParameterLists

  def run_get_task(working_dir, tool: GET_TASK)
    output = nil
    status = nil
    Dir.chdir(working_dir) do
      cmd = windows? ? ['ruby', tool] : [tool]
      output = IO.popen(GEM_ENV, cmd, &:read)
      status = $CHILD_STATUS
    end
    [status, output]
  end

  def run_start_work(working_dir, tool: START_WORK, task_description: nil, branch_name: nil)
    output = nil
    status = nil
    Dir.chdir(working_dir) do
      cmd = windows? ? ['ruby', tool] : [tool]
      cmd << "--task-description=#{task_description}" if task_description
      cmd << "--branch-name=#{branch_name}" if branch_name
      output = IO.popen(GEM_ENV, cmd, &:read)
      status = $CHILD_STATUS
    end
    [status, output]
  end

  def run_agent_task_setup(working_dir, tool: AGENT_TASK)
    output = nil
    status = nil
    Dir.chdir(working_dir) do
      cmd = windows? ? ['ruby', tool, 'setup'] : [tool, 'setup']
      output = IO.popen(GEM_ENV, cmd, &:read)
      status = $CHILD_STATUS
    end
    [status, output]
  end
end
