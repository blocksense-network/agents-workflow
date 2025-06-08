# frozen_string_literal: true

require 'English'
require 'open3'
require 'fileutils'

# Custom exception classes for VCS operations
class VCSError < StandardError; end
class RepositoryNotFoundError < VCSError; end
class VCSTypeNotFoundError < VCSError; end

class VCSRepo
  attr_reader :root, :vcs_type

  def initialize(path_in_repo = Dir.pwd)
    @root = find_repo_root(path_in_repo)
    raise RepositoryNotFoundError, "Could not find repository root from #{path_in_repo}" unless @root

    @vcs_type = determine_vcs_type(@root)
    raise VCSTypeNotFoundError, "Could not determine VCS type for repository at #{@root}" unless @vcs_type
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
    raise "Error: Invalid branch name '#{branch_name}'" unless valid_branch_name?(branch_name)

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
        system('hg', 'add', file_path)
        system('hg', 'commit', '-m', message, file_path)
      when :bzr
        system("bzr add #{file_path}")
        system("bzr commit -m '#{message}'")
      when :fossil
        system('fossil', 'add', file_path)
        system('fossil', 'commit', '-m', message, '--', file_path)
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
        system('hg', 'push', '--new-branch', '--rev', branch_name)
      when :bzr
        system('bzr push')
      when :fossil
        system('fossil push')
      else
        puts "Error: Unknown VCS type (#{@vcs_type}) to push branch"
      end
    end
  end

  def force_push_current_branch(remote, branch)
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        system('git', 'push', remote, "HEAD:#{branch}", '--force')
      when :hg
        system('hg', 'push', remote, '--force', '-b', branch)
      when :bzr
        system('bzr', 'push', remote, '--overwrite')
      when :fossil
        system('fossil', 'push', remote)
      else
        puts "Error: Unknown VCS type (#{@vcs_type}) to force push"
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

  def create_local_branch(branch_name)
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        system('git', 'checkout', '-b', branch_name)
      when :hg
        system('hg', 'bookmark', branch_name)
        system('hg', 'update', branch_name)
      when :fossil
        system('fossil', 'branch', 'new', branch_name, current_branch)
        system('fossil', 'update', branch_name)
      end
    end
  end

  def default_branch
    case @vcs_type
    when :git
      'main'
    when :hg
      'default'
    when :fossil
      'trunk'
    else
      'main'
    end
  end

  def add_file(file_path)
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        system('git', 'add', file_path)
      when :hg
        system('hg', 'add', file_path)
      when :fossil
        system('fossil', 'add', file_path)
      end
    end
  end

  def working_copy_status
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        `git status --porcelain`
      when :hg
        `hg status`
      when :fossil
        `fossil changes`
      else
        ''
      end
    end.strip
  end

  def tip_commit(branch = current_branch)
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        `git rev-parse #{branch}`.strip
      when :hg
        commit = `hg log -r #{branch} --template '{node}'`.strip
        commit = commit[1..-2] if commit.start_with?("'") && commit.end_with?("'")
        commit
      when :fossil
        escaped = branch.gsub("'", "''")
        sql = 'SELECT blob.uuid FROM tag JOIN tagxref ON tag.tagid=tagxref.tagid ' \
              "JOIN blob ON blob.rid=tagxref.rid WHERE tag.tagname='sym-#{escaped}' " \
              'ORDER BY tagxref.mtime DESC LIMIT 1'
        `fossil sql "#{sql}"`.strip.gsub("'", '')
      else
        ''
      end
    end
  end

  def commit_count(base_branch, branch)
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        `git rev-list #{base_branch}..#{branch} --count`.to_i
      when :hg
        revset = "branch(#{branch}) and not ancestors(#{base_branch})"
        out, = Open3.capture2('hg', 'log', '-r', revset, '--template', '{node}\n')
        out.lines.count
      when :fossil
        escaped = branch.gsub("'", "''")
        sql = 'SELECT count(*) FROM tag JOIN tagxref ON tag.tagid=tagxref.tagid WHERE ' \
              "tag.tagname='sym-#{escaped}'"
        count = `fossil sql "#{sql}"`.to_i
        [count - 1, 0].max
      else
        0
      end
    end
  end

  def branch_exists?(branch_name)
    branches.include?(branch_name)
  end

  def branches
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        `git branch --list`.split("\n").map(&:strip)
      when :hg
        `hg branches`.lines.map { |l| l.split.first }
      when :fossil
        `fossil branch list`.lines.map { |l| l.sub(/^[ *]+/, '').strip }
      else
        []
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
            if $CHILD_STATUS.success? && !merge_base_commit.empty?
              commit_hash = `git log --reverse --pretty=%H #{merge_base_commit}..HEAD | head -n 1`.strip
            end
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
          escaped_branch = current_fossil_branch.gsub("'", "''")
          sql = 'SELECT blob.uuid FROM tag JOIN tagxref ON tag.tagid=tagxref.tagid ' \
                'JOIN event ON tagxref.rid=event.objid JOIN blob ON blob.rid=tagxref.rid ' \
                "WHERE tag.tagname='sym-#{escaped_branch}' " \
                "AND event.comment NOT LIKE 'Create new branch named%' " \
                'ORDER BY tagxref.mtime ASC LIMIT 1'
          commit_hash = `fossil sql "#{sql}"`.strip.delete("'")
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
        output = `hg status --change #{commit_hash} --no-status`.strip
        files = output.split("\n") if $CHILD_STATUS.success? && !output.empty?
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
        sql = 'SELECT filename.name FROM filename JOIN mlink ON filename.fnid=mlink.fnid ' \
              "JOIN blob ON mlink.mid=blob.rid WHERE blob.uuid='#{commit_hash}'"
        output = `fossil sql "#{sql}"`.strip
        files = output.split("\n").map { |line| line.delete("'") } if $CHILD_STATUS.success?
      else
        puts "Error: Unknown VCS type (#{@vcs_type}) to list files in commit"
      end
    end
    files.compact
         .map { |f| f.to_s.strip.tr('\\', '/') }
         .reject(&:empty?)
         .uniq
  end

  # Get the HTTP URL of the default remote (origin)
  # Converts SSH URLs to HTTP format for use in authentication with tokens
  def default_remote_http_url
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        # Get the URL of the 'origin' remote
        url = `git remote get-url origin 2>/dev/null`.strip
        return nil if !$CHILD_STATUS.success? || url.empty?

        # Convert SSH URL to HTTPS format
        # SSH formats to convert:
        # - git@github.com:user/repo.git -> https://github.com/user/repo.git
        # - ssh://git@github.com/user/repo.git -> https://github.com/user/repo.git
        # - ssh://git@gitlab.com:2222/user/repo.git -> https://gitlab.com/user/repo.git
        if url.match?(/^git@([^:]+):(.+)$/)
          # Standard SSH format: git@host:path
          url.sub(/^git@([^:]+):/, 'https://\1/')
        elsif (match = url.match(%r{^ssh://git@([^/]+?)(?::\d+)?/(.+)$}))
          # SSH protocol format: ssh://git@host[:port]/path
          "https://#{match[1]}/#{match[2]}"
        else
          url
        end
      when :hg
        # Get the default remote URL for Mercurial
        url = `hg paths default 2>/dev/null`.strip
        return nil if !$CHILD_STATUS.success? || url.empty?

        url
      when :bzr
        # Get the parent branch URL for Bazaar
        url = `bzr config parent_location 2>/dev/null`.strip
        return nil if !$CHILD_STATUS.success? || url.empty?

        url
      when :fossil
        # Fossil doesn't have a direct equivalent to Git remotes
        # Return the configured sync URL if available
        url = `fossil remote 2>/dev/null | head -n 1`.strip
        return nil if !$CHILD_STATUS.success? || url.empty?

        url
      end
    end
  end

  # Get the commit message for a specific commit hash
  def commit_message(commit_hash)
    return nil if commit_hash.nil? || commit_hash.empty?

    Dir.chdir(@root) do
      case @vcs_type
      when :git
        message = `git log -1 --pretty=format:%B #{commit_hash} 2>/dev/null`.strip
        $CHILD_STATUS.success? ? message : nil
      when :hg
        message = `hg log -r #{commit_hash} --template "{desc}" 2>/dev/null`.strip
        $CHILD_STATUS.success? ? message : nil
      when :bzr
        message = `bzr log -r #{commit_hash} --show-ids 2>/dev/null | grep -A 1000 'message:' | tail -n +2`.strip
        $CHILD_STATUS.success? ? message : nil
      when :fossil
        sql = <<~SQL
          SELECT event.comment FROM event
          JOIN blob ON event.objid = blob.rid
          WHERE blob.uuid='#{commit_hash}' AND event.type='ci'
          LIMIT 1
        SQL
        output = `fossil sql "#{sql.strip}" 2>/dev/null`
        $CHILD_STATUS.success? ? output.strip.delete("'") : nil
      end
    end
  end

  # Find the most recent commit in the current branch that starts with
  # 'Start-Agent-Branch:'. Returns the commit hash or nil if not found.
  def latest_agent_branch_commit
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        # First try to find in current branch lineage
        current_branch_commit = `git log HEAD -E --grep='^Start-Agent-Branch:' -n 1 --pretty=%H`.strip
        return current_branch_commit unless current_branch_commit.empty?

        # If not found in current branch, search across all branches
        # This handles the case where we're checking from a parent directory
        # and the repository might be on the wrong branch
        `git log --all -E --grep='^Start-Agent-Branch:' -n 1 --pretty=%H`.strip
      when :hg
        revset = "reverse(grep('^Start-Agent-Branch:'))"
        out, = Open3.capture2('hg', 'log', '-r', revset, '--limit', '1', '--template', '{node}\n')
        out.strip
      when :fossil
        sql = <<~SQL
          SELECT blob.uuid FROM event JOIN blob ON event.objid=blob.rid
          WHERE event.type='ci' AND event.comment LIKE 'Start-Agent-Branch:%'
          ORDER BY event.mtime DESC LIMIT 1
        SQL
        `fossil sql "#{sql.strip}"`.strip.delete("'")
      else
        ''
      end
    end
  end

  def setup_autopush(target_remote_url, target_branch)
    Dir.chdir(@root) do
      case @vcs_type
      when :git
        author_name = `git log -1 --pretty=format:%an`.strip
        author_email = `git log -1 --pretty=format:%ae`.strip
        system('git', 'config', '--local', 'user.name', author_name) unless author_name.empty?
        system('git', 'config', '--local', 'user.email', author_email) unless author_email.empty?
        if !`git remote`.include?('target_remote')
          system('git', 'remote', 'add', 'target_remote', target_remote_url)
        end
        hook = File.join(@root, '.git', 'hooks', 'post-commit')
        File.write(hook, "#!/bin/sh\ngit push target_remote HEAD:#{target_branch} --force\n")
        File.chmod(0o755, hook)
      when :hg
        author = `hg log -r . --template '{author}'`.strip
        unless author.empty?
          hgrc_path = File.join(@root, '.hg', 'hgrc')
          hgrc_content = File.exist?(hgrc_path) ? File.read(hgrc_path) : ''
          unless hgrc_content.match(/^\[ui\].*?^username\s*=/m)
            if hgrc_content.include?('[ui]')
              hgrc_content.gsub!(/(\[ui\].*?)$/m, "\\1\nusername = #{author}")
            else
              hgrc_content += "\n[ui]\nusername = #{author}\n"
            end
            File.write(hgrc_path, hgrc_content)
          end
        end
        hgrc_path = File.join(@root, '.hg', 'hgrc')
        hgrc_content = File.exist?(hgrc_path) ? File.read(hgrc_path) : ''
        unless hgrc_content.match(/^target_remote\s*=/m)
          if hgrc_content.include?('[paths]')
            hgrc_content.gsub!(/(\[paths\].*?)$/m, "\\1\ntarget_remote = #{target_remote_url}")
          else
            hgrc_content += "\n[paths]\ntarget_remote = #{target_remote_url}\n"
          end
          File.write(hgrc_path, hgrc_content)
        end
        hgrc_path = File.join(@root, '.hg', 'hgrc')
        hgrc_content = File.exist?(hgrc_path) ? File.read(hgrc_path) : ''
        hook_command = "hg push target_remote -b #{target_branch} 2>/dev/null || true"
        if hgrc_content.include?('[hooks]')
          unless hgrc_content.match(/^commit\s*=/m)
            hgrc_content.gsub!(/(\[hooks\].*?)$/m, "\\1\ncommit = #{hook_command}")
            File.write(hgrc_path, hgrc_content)
          end
        else
          hgrc_content += "\n[hooks]\ncommit = #{hook_command}\n"
          File.write(hgrc_path, hgrc_content)
        end
      when :bzr
        author_output = `bzr log -l 1 --show-ids | grep committer:`.strip
        unless author_output.empty?
          current_email = begin
            `bzr whoami`.strip
          rescue StandardError
            ''
          end
          if current_email.empty? && (author_output =~ /committer:\s*(.+)/)
            system('bzr', 'whoami', ::Regexp.last_match(1).strip)
          end
        end
        branch_conf_path = File.join(@root, '.bzr', 'branch', 'branch.conf')
        if File.exist?(branch_conf_path)
          conf_content = File.read(branch_conf_path)
          unless conf_content.include?('push_location')
            conf_content += "push_location = #{target_remote_url}\n"
            File.write(branch_conf_path, conf_content)
          end
        else
          FileUtils.mkdir_p(File.dirname(branch_conf_path))
          File.write(branch_conf_path, "push_location = #{target_remote_url}\n")
        end
        hooks_dir = File.join(@root, '.bzr', 'hooks')
        FileUtils.mkdir_p(hooks_dir)
        hook_file = File.join(hooks_dir, 'post_commit.py')
        hook_content = <<~PYTHON
          #!/usr/bin/env python
          import subprocess
          import sys

          def post_commit(local, master, old_revno, old_revid, new_revno, new_revid):
              try:
                  subprocess.call(['bzr', 'push', '--quiet'],
                                cwd='#{@root}',
                                stdout=subprocess.DEVNULL,
                                stderr=subprocess.DEVNULL)
              except:
                  pass
        PYTHON
        File.write(hook_file, hook_content)
        File.chmod(0o755, hook_file)
      when :fossil
        author = `fossil sql 'SELECT user FROM event ORDER BY mtime DESC LIMIT 1'`.strip
        system('fossil', 'user', 'default', author.delete("'")) unless author.empty? || author == 'root'
        current_remote = begin
          `fossil remote`.strip
        rescue StandardError
          ''
        end
        system('fossil', 'remote', 'add', 'target_remote', target_remote_url) if current_remote.empty?
        # Using autosync in fossil provides the desired behavior
        # (automatically push commits to the target remote)
        begin
          system('fossil', 'set', 'autosync', 'on')
        rescue StandardError
          nil
        end
      else
        puts "Error: Unknown VCS type (#{@vcs_type}) to prepare work environment"
      end
    end
  end

  private

  # The branch name validation is intentionally simple and mirrors the common
  # subset of rules for Git and Mercurial. See the Git reference format
  # documentation[1] and the Mercurial branch naming guide[2] for the full
  # specifications.
  #
  # [1]: https://git-scm.com/docs/git-check-ref-format
  # [2]: https://www.mercurial-scm.org/repo/hg/help/branches
  def valid_branch_name?(name)
    !!(name =~ /\A[a-zA-Z0-9._-]+\z/)
  end

  def find_repo_root(start_path)
    current_dir = File.expand_path(start_path)
    current_dir = File.dirname(current_dir) unless File.directory?(current_dir)

    until current_dir == '/' ||
          Dir.exist?(File.join(current_dir, '.git')) ||
          Dir.exist?(File.join(current_dir, '.hg')) ||
          Dir.exist?(File.join(current_dir, '.bzr')) ||
          File.exist?(File.join(current_dir, '.fslckout')) ||
          File.exist?(File.join(current_dir, '_FOSSIL_'))
      parent_dir = File.expand_path('..', current_dir)
      break if parent_dir == current_dir

      current_dir = parent_dir
    end

    if Dir.exist?(File.join(current_dir, '.git')) ||
       Dir.exist?(File.join(current_dir, '.hg')) ||
       Dir.exist?(File.join(current_dir, '.bzr')) ||
       File.exist?(File.join(current_dir, '.fslckout')) ||
       File.exist?(File.join(current_dir, '_FOSSIL_'))
      return current_dir
    end

    nil
  end

  def determine_vcs_type(root_path)
    return :git if Dir.exist?(File.join(root_path, '.git'))
    return :hg if Dir.exist?(File.join(root_path, '.hg'))
    return :bzr if Dir.exist?(File.join(root_path, '.bzr'))
    return :fossil if File.exist?(File.join(root_path, '.fslckout')) ||
                      File.exist?(File.join(root_path, '_FOSSIL_'))

    nil
  end
end
