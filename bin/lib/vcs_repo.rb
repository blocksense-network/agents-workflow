# frozen_string_literal: true

require 'English'
require 'open3'

class VCSRepo
  attr_reader :root, :vcs_type

  def initialize(path_in_repo = Dir.pwd)
    @root = find_repo_root(path_in_repo)
    raise "Error: Could not find repository root from #{path_in_repo}" unless @root

    @vcs_type = determine_vcs_type(@root)
    raise "Error: Could not determine VCS type for repository at #{@root}" unless @vcs_type
  end

  def current_branch
    branch = nil
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        branch = `git rev-parse --abbrev-ref HEAD`.strip
      when :hg
        branch = `hg branch`.strip
      when :bzr
        branch = `bzr nick`.strip
      when :fossil
        branch = `fossil branch list | grep '*' | sed 's/* //'`.strip
      end
    end
    return branch if branch && !branch.empty?

    puts "Error: Could not determine branch in repository at #{@root}"
    exit 1
  end

  def start_branch(branch_name)
    require 'open3'
    output = ''
    status = nil
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        output, status = Open3.capture2e('git', 'checkout', '-b', branch_name)
      when :hg
        output, status = Open3.capture2e('hg', 'branch', branch_name)
      when :bzr
        output, status = Open3.capture2e('bzr', 'switch', '-b', branch_name)
      when :fossil
        output, status = Open3.capture2e('fossil', 'branch', 'new', branch_name, 'trunk')
        if status.success?
          extra, status = Open3.capture2e('fossil', 'update', branch_name)
          output << extra unless status.success?
        end
      else
        raise "Error: Unknown VCS type (#{@vcs_type}) to start branch"
      end
    end
    return if status&.success?

    message = output.strip.empty? ? "Error: Failed to create branch '#{branch_name}'" : output.strip
    raise message
  end

  def commit_file(file_path, message)
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        system('git', 'add', '--', file_path)
        system('git', 'commit', '-m', message, '--', file_path)
      when :hg
        system("hg add #{file_path}")
        system("hg commit -m '#{message}'")
      when :bzr
        system("bzr add #{file_path}")
        system("bzr commit -m '#{message}'")
      when :fossil
        system("fossil add #{file_path}")
        system("fossil commit -m '#{message}'")
      else
        puts "Error: Unknown VCS type (#{@vcs_type}) to commit file"
      end
    end
  end

  def push_current_branch(branch_name, remote = 'origin')
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        system("git push -u #{remote} #{branch_name}")
      when :hg
        system("hg push --rev #{branch_name}")
      when :bzr
        system('bzr push')
      when :fossil
        system('fossil push')
      else
        puts "Error: Unknown VCS type (#{@vcs_type}) to push branch"
      end
    end
  end

  def checkout_branch(branch_name)
    return unless branch_name && !branch_name.empty?

    Dir.chdir(@root) do
      case @vcs_type
      when :git
        system("git checkout #{branch_name}")
      when :hg
        system("hg update #{branch_name}")
      when :bzr
        system("bzr switch #{branch_name}")
      when :fossil
        system("fossil update #{branch_name}")
      else
        puts "Error: Unknown VCS type (#{@vcs_type}) to checkout branch"
      end
    end
  end

  def first_commit_in_current_branch
    # Find the first commit that belongs to the current branch.
    #
    # If we are on the main development branch ("main" or "master") the answer
    # is simply the root commit. For feature branches we find the merge base with
    # the branch's upstream (if configured) or fall back to the primary branch
    # and then return the first commit after that point.
    commit_hash = nil
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        current_branch_name = `git rev-parse --abbrev-ref HEAD`.strip

        # Detect whether the current branch is one of the primary branches.
        is_primary_branch = false
        if current_branch_name == 'main' && system('git rev-parse --verify --quiet refs/heads/main',
                                                   err: File::NULL, out: File::NULL)
          is_primary_branch = true
        elsif current_branch_name == 'master' && system('git rev-parse --verify --quiet refs/heads/master',
                                                        err: File::NULL, out: File::NULL)
          is_primary_branch = true
        end

        if is_primary_branch
          # The command can sometimes output "commit <hash>\n<hash>". We need the last line.
          commit_hash = `git rev-list --max-parents=0 HEAD --pretty=%H`.lines.last&.strip
        else
          # For feature branches try to determine the branch point.
          base_branch_ref = nil

          # Start with the configured upstream branch. Some branches may track
          # themselves, so ignore the upstream if it points at the current commit.
          upstream = `git rev-parse --abbrev-ref @{u} 2>/dev/null`.strip
          if $CHILD_STATUS.success? && !upstream.empty? && system("git rev-parse --verify --quiet #{upstream}^{commit}",
                                                                  err: File::NULL, out: File::NULL)
            upstream_commit = `git rev-parse #{upstream}`.strip
            head_commit = `git rev-parse HEAD`.strip
            base_branch_ref = upstream unless upstream_commit == head_commit
          end

          # Fallback to main or master if we couldn't use the upstream.
          if base_branch_ref.nil?
            if system('git rev-parse --verify --quiet refs/heads/main^{commit}', err: File::NULL, out: File::NULL)
              base_branch_ref = 'main'
            elsif system('git rev-parse --verify --quiet refs/heads/master^{commit}', err: File::NULL,
                                                                                      out: File::NULL)
              base_branch_ref = 'master'
            end
          end

          if base_branch_ref
            merge_base_commit = `git merge-base #{base_branch_ref} HEAD`.strip
            commit_hash = `git log --reverse --pretty=%H #{merge_base_commit}..HEAD | head -n 1`.strip if $CHILD_STATUS.success? && !merge_base_commit.empty?
          end
        end
      when :hg
        current_hg_branch = `hg branch`.strip
        commit_hash = `hg log -r "min(branch('#{current_hg_branch}'))" --template "{node}\\n"`.strip
      when :bzr
        current_bzr_branch = `bzr nick .`.strip
        # Ensure branch name is not empty and does not contain problematic characters for revset
        if current_bzr_branch && !current_bzr_branch.empty? && current_bzr_branch.match?(/\A[a-zA-Z0-9._-]+\z/)
          commit_hash = `bzr log -r "first(branch('#{current_bzr_branch}'))" --format=rev_id`.strip
        end
      when :fossil
        current_fossil_branch = `fossil branch current`.strip
        # Ensure branch name is not empty and is safe for SQL query
        if current_fossil_branch && !current_fossil_branch.empty? && current_fossil_branch.match?(/\A[a-zA-Z0-9._-]+\z/)
          # Fossil branch names can contain characters that need escaping in SQL,
          # but typically they are simple. Using as is, assuming simple names.
          # For more complex names, proper SQL escaping would be needed.
          commit_hash = `fossil sql "SELECT lower(hex(blob.uuid)) FROM event JOIN blob ON event.objid=blob.rid WHERE event.type='ci' AND event.branch = '#{current_fossil_branch.gsub(
            "'", "''"
          )}' ORDER BY event.mtime ASC LIMIT 1"`.strip
        end
      else
        puts "Error: Unknown VCS type (#{@vcs_type}) to find first commit"
        # No exit here, will return nil by default
      end
    end
    return commit_hash if commit_hash && !commit_hash.empty?

    nil
  end

  def files_in_commit(commit_hash)
    return [] if commit_hash.nil? || commit_hash.empty?

    files = []
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        # For git, this shows files changed in the specified commit
        output = `git diff-tree --no-commit-id --name-only -r #{commit_hash}`.strip
        files = output.split("\n") if $CHILD_STATUS.success?
      when :hg
        # For hg, this lists files modified in the specified changeset
        output = `hg status --rev #{commit_hash} --print0 --no-status --added --modified --removed`.strip
        # --print0 uses null byte as separator
        files = output.split("\0") if $CHILD_STATUS.success? && !output.empty?
      when :bzr
        # For bzr, whatchanged shows files modified in the revision.
        # We need to parse its output. It lists files under "added:", "removed:", "modified:".
        # A simpler approach might be `bzr diff -c #{commit_hash} --short`, but that might include content.
        # `bzr whatchanged -r #{commit_hash}` output is like:
        # ---
        # revno: 123
        # author: ...
        # ...
        # modified:
        #   foo.txt
        # added:
        #   bar.txt
        # ---
        # This is complex to parse robustly.
        # Let's try `bzr ls -r #{commit_hash} --kind=file` for files in revision,
        # or `bzr diff --short -r #{commit_hash}^..#{commit_hash}` if parent ref `^` works.
        # `bzr version-info --show-diff <rev>` might be better.
        # `bzr whatchanged -r #{commit_hash} --short` lists files prefixed by status (A, M, D)
        output = `bzr whatchanged -r #{commit_hash} --short`.strip
        if $CHILD_STATUS.success?
          output.split("\n").each do |line|
            # Example lines: "A  path/to/file.txt", "M  another.txt"
            # We just need the file path part.
            # Regex to capture file path after status char and spaces.
            match = line.match(/^[A-Z]\s+(.*)$/)
            files << match[1] if match && match[1]
          end
        end
      when :fossil
        # For fossil, 'fossil changes --hash HASH --name-only' lists files changed in that commit.
        # Or 'fossil finfo -b HASH' lists all files in that checkin.
        # 'fossil diff --name-only --from HASH^ --to HASH'
        # The simplest seems to be 'changes'
        output = `fossil changes --hash #{commit_hash} --name-only --no-header`.strip
        files = output.split("\n") if $CHILD_STATUS.success?
      else
        puts "Error: Unknown VCS type (#{@vcs_type}) to list files in commit"
      end
    end
    files.compact.map(&:strip).reject(&:empty?).uniq
  end

  private

  def find_repo_root(start_path)
    current_dir = File.expand_path(start_path)
    current_dir = File.dirname(current_dir) unless File.directory?(current_dir)

    until current_dir == '/' ||
          Dir.exist?(File.join(current_dir, '.git')) ||
          Dir.exist?(File.join(current_dir, '.hg')) ||
          Dir.exist?(File.join(current_dir, '.bzr')) ||
          Dir.exist?(File.join(current_dir, '.fossil'))
      parent_dir = File.expand_path('..', current_dir)
      break if parent_dir == current_dir

      current_dir = parent_dir
    end

    if Dir.exist?(File.join(current_dir, '.git')) ||
       Dir.exist?(File.join(current_dir, '.hg')) ||
       Dir.exist?(File.join(current_dir, '.bzr')) ||
       Dir.exist?(File.join(current_dir, '.fossil'))
      return current_dir
    end

    nil
  end

  def determine_vcs_type(root_path)
    return :git if Dir.exist?(File.join(root_path, '.git'))
    return :hg if Dir.exist?(File.join(root_path, '.hg'))
    return :bzr if Dir.exist?(File.join(root_path, '.bzr'))
    return :fossil if Dir.exist?(File.join(root_path, '.fossil'))

    nil
  end
end
