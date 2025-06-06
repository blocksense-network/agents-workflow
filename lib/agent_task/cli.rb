# frozen_string_literal: true

module AgentTask
  # CLI exposes the main binaries as callable methods so the functionality
  # can be reused programmatically. The methods here mirror the behavior
  # of the command line tools.
  module CLI # rubocop:disable Metrics/ModuleLength
    module_function

    EDITOR_HINT = <<~HINT
      # Please write your task prompt above.
      # Enter an empty prompt to abort the task creation process.
      # Feel free to leave this comment in the file. It will be ignored.
    HINT

    # Implements the same workflow as the `agent-task` executable.
    # Arguments should match the command line invocation.
    def start_task(args, stdin: $stdin, stdout: $stdout)
      require 'tempfile'
      require 'fileutils'
      require 'time'
      require 'optparse'
      require_relative '../vcs_repo'
      require_relative '../agent_tasks'

      options = {}
      OptionParser.new do |opts|
        opts.on('--push-to-remote=BOOL', 'Push branch to remote automatically') do |val|
          options[:push_to_remote] = val
        end
        opts.on('--prompt=STRING', 'Use STRING as the task prompt') do |val|
          options[:prompt] = val
        end
        opts.on('--prompt-file=FILE', 'Read the task prompt from FILE') do |val|
          options[:prompt_file] = val
        end
      end.parse!(args)

      branch_name = args.shift
      start_new_branch = branch_name && !branch_name.strip.empty?
      abort('Error: --prompt and --prompt-file are mutually exclusive') if options[:prompt] && options[:prompt_file]

      prompt_content = nil
      if options[:prompt]
        prompt_content = options[:prompt].dup
      elsif options[:prompt_file]
        begin
          prompt_content = File.read(options[:prompt_file])
        rescue StandardError => e
          abort("Error: Failed to read prompt file: #{e.message}")
        end
      end

      begin
        repo = VCSRepo.new
      rescue StandardError => e
        stdout.puts e.message
        exit 1
      end

      orig_branch = repo.current_branch
      if start_new_branch
        begin
          repo.start_branch(branch_name)
        rescue StandardError => e
          stdout.puts e.message
          exit 1
        end
      else
        branch_name = orig_branch
        main_names = [repo.default_branch, 'main', 'master', 'trunk', 'default']
        abort('Error: Refusing to run on the main branch') if main_names.include?(branch_name)
      end

      cleanup_branch = start_new_branch

      begin
        task_content = nil
        if prompt_content.nil?
          tempfile = Tempfile.new(['task', '.txt'])
          tempfile.write("\n")
          tempfile.write(EDITOR_HINT)
          tempfile.close

          editor = ENV.fetch('EDITOR', nil)
          unless editor
            editors = %w[nano pico micro vim helix vi]
            editors.each do |ed|
              if system("command -v #{ed} > /dev/null 2>&1")
                editor = ed
                break
              end
            end
            editor ||= 'nano'
          end

          abort('Error: Failed to open the editor.') unless system("#{editor} #{tempfile.path}")
          task_content = File.read(tempfile.path)
          task_content.sub!("\n#{EDITOR_HINT}", '')
          task_content.sub!(EDITOR_HINT, '')
        else
          task_content = prompt_content
        end
        task_content.gsub!("\r\n", "\n")
        abort('Aborted: empty task prompt.') if task_content.strip.empty?

        if start_new_branch
          now = Time.now.utc
          year = now.year
          month = format('%02d', now.month)
          day = format('%02d', now.day)
          hour = format('%02d', now.hour)
          min = format('%02d', now.min)
          filename = "#{day}-#{hour}#{min}-#{branch_name}"
          tasks_dir = File.join(repo.root, '.agents', 'tasks', year.to_s, month)
          FileUtils.mkdir_p(tasks_dir)
          task_file = File.join(tasks_dir, filename)

          commit_msg = "Start-Agent-Branch: #{branch_name}"
          target_remote = repo.default_remote_http_url
          commit_msg += "\nTarget-Remote: #{target_remote}" if target_remote

          File.binwrite(task_file, task_content)
          repo.commit_file(task_file, commit_msg)
        else
          start_commit = repo.latest_agent_branch_commit
          abort('Error: Could not locate task start commit') unless start_commit
          files = repo.files_in_commit(start_commit)
          abort('Error: Task start commit should introduce exactly one file') unless files.length == 1
          task_file = File.join(repo.root, files.first)
          File.open(task_file, 'a') do |f|
            f.write("\n--- FOLLOW UP TASK ---\n")
            f.write(task_content)
          end
          repo.commit_file(task_file, 'Follow-up task')
        end

        push = nil
        if options.key?(:push_to_remote)
          val = options[:push_to_remote].to_s.downcase
          truthy = %w[1 true yes y].include?(val)
          falsy = %w[0 false no n].include?(val)
          abort("Error: Invalid value for --push-to-remote: '#{options[:push_to_remote]}'") unless truthy || falsy
          push = truthy
        else
          stdout.print 'Push to default remote? [Y/n]: '
          input = stdin.gets
          abort('Error: Non-interactive environment, use --push-to-remote option.') if input.nil?
          answer = input.strip
          answer = 'y' if answer.empty?
          push = answer.downcase.start_with?('y')
        end
        repo.push_current_branch(branch_name) if push

        cleanup_branch = false
      ensure
        repo.checkout_branch(orig_branch) if orig_branch
        if cleanup_branch
          case repo.vcs_type
          when :git
            system('git', 'branch', '-D', branch_name, chdir: repo.root, out: File::NULL, err: File::NULL)
          when :fossil
            system('fossil', 'branch', 'close', branch_name, chdir: repo.root, out: File::NULL, err: File::NULL)
          end
        end
      end
    end

    # Print the current task description, replicating the `get-task` command.
    def run_get_task(args = [])
      require 'resolv'
      require 'fileutils'
      require 'optparse'
      require_relative '../vcs_repo'
      require_relative '../agent_tasks'

      options = {}
      OptionParser.new do |opts|
        opts.on('--autopush', 'Tells the agent to automatically push its changes') do
          options[:autopush] = true
        end
      end.parse!(args)

      begin
        retriever = AgentTasks.new
        puts retriever.agent_prompt(autopush: options[:autopush])
        return
      rescue RepositoryNotFoundError
        # Expected when current directory is not in a VCS repo - continue to scan subdirectories
      end

      dir_messages = []
      Dir.children(Dir.pwd).sort.each do |entry|
        candidate = File.join(Dir.pwd, entry)
        next unless File.directory?(candidate)

        vcs_dirs = %w[.git .hg .bzr .fslckout _FOSSIL_]
        next unless vcs_dirs.any? { |v| File.exist?(File.join(candidate, v)) || Dir.exist?(File.join(candidate, v)) }

        begin
          msg = AgentTasks.new(candidate).agent_prompt(autopush: options[:autopush])
          dir_messages << [entry, msg] if msg && !msg.empty?
        rescue StandardError
          # Ignore repositories without an active agent task
        end
      end

      if dir_messages.empty?
        puts "Error: Could not find repository root from #{Dir.pwd}"
        exit 1
      elsif dir_messages.length == 1
        puts dir_messages[0][1]
      else
        output = dir_messages.map { |dir, msg| "In directory `#{dir}`:\n#{msg}" }.join("\n\n")
        puts output
      end
    rescue StandardError => e
      puts e.message
      exit 1
    end
  end
end
