# frozen_string_literal: true

require_relative 'vcs_repo'
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

  def on_task_branch?; end

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

  def git_details
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

    if target_remote.start_with?('https://github.com/')
      github_token = ENV.fetch('GITHUB_ACCESS_TOKEN', nil)
      unless github_token
        raise StandardError,
              'Error: The Codex environment must be configured with a GITHUB_ACCESS_TOKEN, specified as a secret'
      end

      remote_url = target_remote.sub('https://github.com/', "https://x-access-token:#{github_token}@github.com/")
    else
      remote_url = target_remote
    end

    { remote_url: remote_url, push_branch: target_branch }
  end

  def prepare_work_environment
    details = git_details
    Dir.chdir(@repo.root) do
      author_name = `git log -1 --pretty=format:%an`.strip
      author_email = `git log -1 --pretty=format:%ae`.strip
      system('git', 'config', '--local', 'user.name', author_name)
      system('git', 'config', '--local', 'user.email', author_email)
      system('git', 'remote', 'add', 'target_remote', details[:remote_url])
    end
  end

  def agent_prompt(autopush: false)
    task_file_contents = File.read(agent_task_file_in_current_branch)
    tasks = task_file_contents.split("\n--- FOLLOW UP TASK ---\n")
    if tasks.length == 1
      message = tasks[0]
    else
      message = ''
      tasks.each_with_index do |task_text, index|
        message += if index.zero?
                     "You were given the following task:\n#{task_text}\n"
                   elsif index == tasks.length - 1
                     "Your current task is:\n#{task_text}\n"
                   else
                     "You were given a follow-up task:\n#{task_text}\n"
                   end
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

    if system('which nix > /dev/null 2>&1')
      message += <<~NIX_MESSAGE

        # Appendix (Using Nix)

        Since Nix is available in your PATH, you can discover the paths to
        all Nix dependencies by examining the current environment variables.
        This can be helpful for finding documentation, source code, or other
        resources that are part of your Nix environment.
      NIX_MESSAGE
    end

    if autopush
      # Extract target remote and branch from the task commit message
      first_commit_hash = @repo.latest_agent_branch_commit
      raise StandardError, 'Error: Could not find first commit in current branch' unless first_commit_hash

      commit_msg = @repo.commit_message(first_commit_hash)
      raise StandardError, 'Error: Could not retrieve commit message from first commit' unless commit_msg

      # Extract Target-Remote from commit message
      remote_match = commit_msg.match(/^Target-Remote:\s*(.+?)$/m)
      raise StandardError, 'Error: Target-Remote not found in commit message' unless remote_match

      target_remote = remote_match[1].strip
      raise StandardError, 'Error: Target-Remote is empty in commit message' if target_remote.empty?

      # Extract branch name from Start-Agent-Branch line
      branch_match = commit_msg.match(/^Start-Agent-Branch:\s*(.+?)$/m)
      raise StandardError, 'Error: Start-Agent-Branch not found in commit message' unless branch_match

      target_branch = branch_match[1].strip
      raise StandardError, 'Error: Start-Agent-Branch is empty in commit message' if target_branch.empty?

      details = git_details
      push_branch = details[:push_branch]
      message += <<~GIT_MESSAGE

        # Appendix (Using Git)

        Before starting work, execute the `start-work` command.

        Once you complete your task, examine all the changes that you've made
        and squash your work in a single commit.

        Make sure that the commit message includes a summary of your changes.

        Finally, push your commit with the following command:

        git push target_remote HEAD:#{push_branch}
      GIT_MESSAGE
    end

    message
  end
end
