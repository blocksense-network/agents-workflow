# frozen_string_literal: true

require_relative 'vcs_repo'
require_relative 'platform_helpers'
require 'fileutils'
require 'net/http'
require 'uri'

class AgentTasks
  attr_reader :repo

  def initialize(path_in_repo = Dir.pwd)
    @repo = VCSRepo.new(path_in_repo) # This can raise if repo is not found
  end

  def agent_task_file_in_current_branch
    start_commit_hash = @repo.latest_agent_branch_commit
    unless start_commit_hash && !start_commit_hash.empty?
      raise StandardError,
            'You are not currently on a agent task branch'
    end

    files_in_commit = @repo.files_in_commit(start_commit_hash)
    if files_in_commit.nil? || files_in_commit.empty?
      raise StandardError,
            "Error: No files found in the task start commit ('#{start_commit_hash}')."
    end

    File.join(@repo.root, files_in_commit.first)
  end

  def on_task_branch?
    !!@repo.latest_agent_branch_commit && !@repo.latest_agent_branch_commit.empty?
  end

  def record_initial_task(task_content, branch_name, devshell: nil)
    now = Time.now.utc
    year = now.year
    month = format('%02d', now.month)
    day = format('%02d', now.day)
    hour = format('%02d', now.hour)
    min = format('%02d', now.min)

    tasks_dir = File.join(@repo.root, '.agents', 'tasks', year.to_s, month)
    FileUtils.mkdir_p(tasks_dir)
    filename = "#{day}-#{hour}#{min}-#{branch_name}"
    task_file = File.join(tasks_dir, filename)

    commit_msg = "Start-Agent-Branch: #{branch_name}"
    target_remote = @repo.default_remote_http_url
    commit_msg += "\nTarget-Remote: #{target_remote}" if target_remote
    commit_msg += "\nDev-Shell: #{devshell}" if devshell

    File.binwrite(task_file, task_content)
    @repo.commit_file(task_file, commit_msg)
  end

  def append_task(task_content)
    start_commit = @repo.latest_agent_branch_commit
    raise StandardError, 'Error: Could not locate task start commit' unless start_commit

    files = @repo.files_in_commit(start_commit)
    raise StandardError, 'Error: Task start commit should introduce exactly one file' unless files.length == 1

    task_file = File.join(@repo.root, files.first)
    File.open(task_file, 'a') do |f|
      f.write("\n--- FOLLOW UP TASK ---\n")
      f.write(task_content)
    end
    @repo.commit_file(task_file, 'Follow-up task')
  end

  def online?
    # Use Google's connectivity check service - a lightweight endpoint designed for connectivity testing
    # This service is globally distributed and operated by Google, making it highly reliable
    # Reference: https://developers.google.com/speed/public-dns/docs/doh
    uri = URI('http://connectivitycheck.gstatic.com/generate_204')

    Net::HTTP.start(uri.host, uri.port, open_timeout: 3, read_timeout: 3) do |http|
      response = http.get(uri.path)
      # Google's connectivity check returns 204 No Content on success
      response.code == '204'
    end
  rescue StandardError
    false
  end

  def setup_autopush
    # Extract target remote and branch from the task commit message
    first_commit_hash = @repo.latest_agent_branch_commit
    raise StandardError, 'Error: Could not find first commit in current branch' unless first_commit_hash

    commit_msg = @repo.commit_message(first_commit_hash)
    raise StandardError, 'Error: Could not retrieve commit message from first commit' unless commit_msg

    remote_match = commit_msg.match(/^Target-Remote:\s*(.+?)$/m)
    raise StandardError, 'Error: Target-Remote not found in commit message' unless remote_match

    target_remote = remote_match[1].strip
    raise StandardError, 'Error: Target-Remote is empty in commit message' if target_remote.empty?

    branch_match = commit_msg.match(/^Start-Agent-Branch:\s*(.+?)$/m)
    raise StandardError, 'Error: Start-Agent-Branch not found in commit message' unless branch_match

    target_branch = branch_match[1].strip
    raise StandardError, 'Error: Start-Agent-Branch is empty in commit message' if target_branch.empty?

    @repo.setup_autopush(target_remote, target_branch)
  end

  def process_workflows(text)
    require 'shellwords'
    require 'open3'
    require 'rbconfig'

    env_vars = {}
    diagnostics = []
    output_lines = []

    text.each_line do |line|
      stripped = line.chomp
      if stripped.start_with?('/')
        tokens = Shellwords.split(stripped[1..])
        cmd = tokens.shift
        wf_dir = File.join(@repo.root, '.agents', 'workflows')
        script = File.join(wf_dir, cmd)
        txt = "#{script}.txt"
        if File.exist?(script)
          unless File.executable?(script)
            begin
              File.chmod(0o755, script)
            rescue StandardError
              diagnostics << "Workflow command '#{cmd}' not executable"
              next
            end
          end
          stdout_str, stderr_str, status = execute_script(script, tokens)
          diagnostics << "$ #{cmd} #{tokens.join(' ')}\n#{stderr_str}" unless status.success?
          stdout_str.each_line { |l| handle_workflow_line(l.chomp, env_vars, diagnostics, output_lines) }
        elsif File.exist?(txt)
          File.read(txt).each_line { |l| handle_workflow_line(l.chomp, env_vars, diagnostics, output_lines) }
        else
          diagnostics << "Unknown workflow command '/#{cmd}'"
        end
      else
        handle_workflow_line(stripped, env_vars, diagnostics, output_lines)
      end
    end

    final_env = {}
    env_vars.each do |var, info|
      values = []
      values.concat(info[:direct].split(',').map(&:strip)) if info[:direct]
      values.concat(info[:append])
      final_env[var] = values.uniq.join(',')
    end

    [output_lines.join("\n"), final_env, diagnostics]
  end

  def handle_workflow_line(line, env_vars, diagnostics, output_lines)
    if line =~ /^@agents-setup\s+(.*)$/
      Shellwords.split(Regexp.last_match(1)).each do |pair|
        op = pair.include?('+=') ? '+=' : '='
        var, val = pair.split(op, 2)
        env_vars[var] ||= { direct: nil, append: [] }
        entry = env_vars[var]
        if op == '='
          if entry[:direct] && entry[:direct] != val
            diagnostics << "Conflicting assignment for #{var}"
          else
            entry[:direct] = val
          end
        else
          entry[:append].concat(val.split(','))
        end
      end
    else
      output_lines << line
    end
  end

  def agent_prompt_with_env
    task_file_contents = File.read(agent_task_file_in_current_branch)
    tasks = task_file_contents.split("\n--- FOLLOW UP TASK ---\n")
    message = ''
    env = {}
    tasks.each_with_index do |task_text, index|
      text, vars, = process_workflows(task_text)
      env.merge!(vars) do |_k, old_v, new_v|
        (old_v.split(',') + new_v.split(',')).uniq.join(',')
      end
      if tasks.length == 1
        message = text
      else
        prefix = if index.zero?
                   "You were given the following task:\n"
                 elsif index == tasks.length - 1
                   "Your current task is:\n"
                 else
                   "You were given a follow-up task:\n"
                 end
        message += "#{prefix}#{text}\n"
      end
    end

    unless online?
      message += <<~OFFLINE_MESSAGE

        # Appendix (Lack of internet access)

        Please note that during development, certain commands will fail because
        you don't have access to the internet.

        All URLs mentioned in the task description(s) have been downloaded
        to the /workspace/internet_resources directory.

        If it's difficult for you to achieve a task without access to additional
        internet resources, you can always propose more URLs that we should make
        available offline.

        Downloading development, dependencies may also fail to download due
        to the lack of internet connectivity. We are trying to maintain the
        script `.agents/build_all_targets.sh` that is also executed before
        your development session starts while your computer is still connected
        to the internet.

        The script tries to run all build commands that have development
        dependencies in order to cache the dependencies for offline use.
        Please propose changes to this script when you introduce new build
        targets with dependencies.

        When you need to consult the documentation or source code modules
        for a particular dependency, always try to find where this dependency
        have been downloaded and try to access the necessary files through
        the file system (i.e. depending on the programming language, the
        operating system and the package manager being used, they should
        be in their standard location).
      OFFLINE_MESSAGE
    end

    if system('which', 'nix', out: File::NULL, err: File::NULL)
      message += <<~NIX_MESSAGE

        # Appendix (Using Nix)

        Since Nix is available in your PATH, you can discover the paths to
        all Nix dependencies by examining the current environment variables.
        This can be helpful for finding documentation, source code, or other
        resources that are part of your Nix environment.
      NIX_MESSAGE
    end

    [message, env]
  end

  def agent_prompt
    msg, = agent_prompt_with_env
    msg
  end

  def agent_prompt_with_autopush_setup(autopush: true)
    setup_autopush if autopush && on_task_branch?
    agent_prompt
  end

  private

  def execute_script(script_path, args)
    require 'open3'

    if windows?
      # On Windows, handle different file extensions and try to use appropriate interpreters
      ext = File.extname(script_path).downcase

      case ext
      when '.bat', '.cmd'
        Open3.capture3('cmd', '/c', script_path, *args)
      when '.rb'
        Open3.capture3('ruby', script_path, *args)
      when '.py'
        Open3.capture3('python', script_path, *args)
      when '.js'
        Open3.capture3('node', script_path, *args)
      when '.ps1'
        Open3.capture3('powershell', '-ExecutionPolicy', 'Bypass', '-File', script_path, *args)
      else
        # For extensionless files or shell scripts, check if bash is available
        if bash_available?
          # Use bash directly for shell scripts
          Open3.capture3('bash', script_path, *args)
        elsif File.exist?(script_path) && File.size(script_path).positive?
          # Fallback: check shebang and try to extract interpreter
          first_line = begin
            File.open(script_path, 'r') { |f| f.readline.chomp }
          rescue StandardError
            ''
          end

          if first_line.start_with?('#!')
            interpreter = extract_interpreter_from_shebang(first_line)
            Open3.capture3(interpreter, script_path, *args)
          else
            fake_status = Struct.new(:success?, :exitstatus).new(false, 1)
            ['', "Cannot execute script on Windows without bash: #{script_path}", fake_status]
          end
        else
          fake_status = Struct.new(:success?, :exitstatus).new(false, 1)
          ['', "Script file not found or empty: #{script_path}", fake_status]
        end
      end
    else
      # On Unix systems, execute normally
      Open3.capture3(script_path, *args)
    end
  end

  def bash_available?
    @bash_available ||= system('bash --version > NUL 2>&1') if windows?
    @bash_available || !windows?
  end

  def extract_interpreter_from_shebang(shebang)
    # Extract interpreter from shebang line
    full_line = shebang.sub(/^#!/, '').strip

    case full_line
    when %r{^/usr/bin/env\s+(.+)}
      ::Regexp.last_match(1).split.first # Handle cases like "/usr/bin/env ruby" or "/usr/bin/env python3"
    when %r{/bin/sh$}, %r{/usr/bin/sh$}
      bash_available? ? 'bash' : 'sh'
    when %r{/bin/bash$}, %r{/usr/bin/bash$}
      bash_available? ? 'bash' : 'sh'
    when /ruby/
      'ruby'
    when /python/
      'python'
    when /node/
      'node'
    else
      # Try to get just the basename
      basename = File.basename(full_line.split.first)
      case basename
      when 'sh', 'bash'
        bash_available? ? 'bash' : 'sh'
      else
        basename
      end
    end
  end
end
