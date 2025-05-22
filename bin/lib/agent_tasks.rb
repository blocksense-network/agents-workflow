# frozen_string_literal: true

require_relative 'vcs_repo'
require 'fileutils'
require 'resolv'

# Renamed from AgentTaskRetriever
class AgentTasks
  def initialize(path_in_repo = Dir.pwd)
    @repo = VCSRepo.new(path_in_repo) # This can raise if repo is not found
  end

  def agent_tasks_in_current_branch
    first_commit_hash = @repo.first_commit_in_current_branch
    raise StandardError, "Error: Could not find the first commit in the current branch '#{@repo.current_branch}'." unless first_commit_hash

    files_in_commit = @repo.files_in_commit(first_commit_hash)
    if files_in_commit.nil? || files_in_commit.empty?
      raise StandardError,
            "Error: No files found in the first commit ('#{first_commit_hash}') of branch '#{@repo.current_branch}'."
    end

    first_file_relative_path = files_in_commit.first
    first_file_absolute_path = File.join(@repo.root, first_file_relative_path)
    agents_dir = File.dirname(first_file_absolute_path)

    unless Dir.exist?(agents_dir)
      raise StandardError, <<~MSG
        Error: Determined task directory #{agents_dir} does not exist.
        (Derived from the first file '#{first_file_relative_path}' in commit '#{first_commit_hash}')
      MSG
    end

    files = Dir.entries(agents_dir).select { |f| f != '.' && f != '..' }
    if files.empty?
      raise StandardError, <<~MSG
        Error: No task files found in the determined task directory #{agents_dir}.
        (Directory derived from the first file '#{first_file_relative_path}' in commit '#{first_commit_hash}')
      MSG
    end

    files.sort.map { |f| File.join(agents_dir, f) }
  end

  def on_task_branch?; end

  def online?
    resolver = Resolv::DNS.new(nameserver: '8.8.8.8', timeout: 3)
    resolver.getaddress('google.com')
    true
  rescue StandardError
    false
  end

  def build_message(task_files)
    message = ''

    if task_files.length == 1
      message = File.read(task_files[0])
    else
      task_files.each_with_index do |file_path, index|
        date = File.basename(file_path)
        content = File.read(file_path)

        message += if index.zero?
                     "On #{date}, you were given the following task:\n#{content}\n"
                   elsif index == task_files.length - 1
                     "Your current task is:\n#{content}\n"
                   else
                     "On #{date}, you were given a follow-up task:\n#{content}\n"
                   end
      end
    end

    unless online?
      message += <<~OFFLINE_MESSAGE
        Please note that during development, certain commands will fail because
        you don't have access to the intenet.

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

    if system('which nix > /dev/null 2>&1')
      message += <<~NIX_MESSAGE
        Since Nix is available in your PATH, you can discover the paths to
        all Nix dependencies by examining the current environment variables.
        This can be helpful for finding documentation, source code, or other
        resources that are part of your Nix environment.
      NIX_MESSAGE
    end

    message
  end

  def agent_prompt
    task_file_paths = agent_tasks_in_current_branch
    build_message(task_file_paths)
  end
end
