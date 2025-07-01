# frozen_string_literal: true

require 'shellwords'
require 'open3'

# WorkflowCommands provides helpers for processing special workflow commands
# embedded in task descriptions. Workflow commands are lines starting with '/'
# followed by a program name and optional arguments. The commands are searched
# under `.agents/workflows` within the repository.
module WorkflowCommands # rubocop:disable Metrics/ModuleLength
  module_function

  # Execute the workflow command or include the matching text file.
  # Returns [output_string, env_hash].
  def run(name, args, repo_root)
    workflows_dir = File.join(repo_root, '.agents', 'workflows')
    script = File.join(workflows_dir, name)
    text = File.join(workflows_dir, "#{name}.txt")

    raise "Unknown workflow command '#{name}'" unless File.exist?(script) || File.exist?(text)

    return [File.read(text), {}] if File.exist?(text)

    unless File.executable?(script)
      begin
        File.chmod(0o755, script)
      rescue StandardError
        # ignore
      end
    end

    raise "Workflow command '#{name}' is not executable" unless File.executable?(script)

    stdout, stderr, status = Open3.capture3(script, *args, chdir: repo_root)
    raise "Error running workflow command '#{name}':\n#{stderr}" unless status.success?

    output, env = extract_env_directives(stdout)
    [output, env]
  end

  # Parse a task description and execute workflow commands. Returns
  # [processed_text, env_hash].
  def process(text, repo_root)
    env = {}
    result_lines = []
    text.each_line do |line|
      if (cmd = parse_command(line))
        name, *args = cmd
        out, sub_env = run(name, args, repo_root)
        check_conflicts!(env, sub_env)
        env.merge!(sub_env)
        out.each_line do |l|
          next if l.strip.start_with?('@agents-setup')

          result_lines << l
        end
      elsif (assigns = parse_env(line))
        check_conflicts!(env, assigns)
        env.merge!(assigns)
      else
        result_lines << line
      end
    end
    [result_lines.join, env]
  end

  # Validate the task description without producing output. Returns array of
  # error messages.
  def validate(text, repo_root)
    env = {}
    errors = []
    text.each_line do |line|
      if (cmd = parse_command(line))
        name, *args = cmd
        begin
          _, sub_env = run(name, args, repo_root)
          begin
            check_conflicts!(env, sub_env)
            env.merge!(sub_env)
          rescue StandardError => e
            errors << e.message
          end
        rescue StandardError => e
          errors << e.message
        end
      elsif (assigns = parse_env(line))
        begin
          check_conflicts!(env, assigns)
          env.merge!(assigns)
        rescue StandardError => e
          errors << e.message
        end
      end
    end
    errors
  end

  def parse_command(line)
    return nil unless line.lstrip.start_with?('/')

    Shellwords.shellsplit(line.strip.sub(%r{^/}, ''))
  rescue ArgumentError
    nil
  end

  def parse_env(line)
    m = line.strip.match(/^@agents-setup\s+(.+)/)
    return nil unless m

    assignments = {}
    Shellwords.shellsplit(m[1]).each do |assign|
      k, v = assign.split('=', 2)
      assignments[k] = v || ''
    end
    assignments
  rescue ArgumentError
    nil
  end

  def check_conflicts!(env, assigns)
    assigns.each do |k, v|
      next unless env.key?(k) && env[k] != v

      raise "Conflicting @agents-setup value for #{k}"
    end
  end

  def extract_env_directives(output)
    env = {}
    lines = []
    output.each_line do |line|
      if (assigns = parse_env(line))
        check_conflicts!(env, assigns)
        env.merge!(assigns)
      else
        lines << line
      end
    end
    [lines.join, env]
  end
end
