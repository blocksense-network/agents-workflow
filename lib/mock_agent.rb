# frozen_string_literal: true

require 'logger'
require 'fileutils'
require 'digest'
require 'time'

# Simple JSON-like output without requiring the JSON gem
# This avoids compatibility issues while maintaining test functionality
module SimpleJSON
  def self.generate(obj)
    case obj
    when Hash
      pairs = obj.map { |k, v| "#{k.inspect}: #{generate(v)}" }
      "{ #{pairs.join(', ')} }"
    when Array
      "[#{obj.map { |v| generate(v) }.join(', ')}]"
    when String
      obj.inspect
    when Numeric, TrueClass, FalseClass, NilClass
      obj.inspect
    else
      obj.to_s.inspect
    end
  end

  def self.pretty_generate(obj)
    generate(obj)
  end
end

# MockAgent simulates realistic AI agent behavior for testing isolation
# and concurrency of the snapshot-based workspace system.
#
# The agent performs realistic file operations that mirror what actual
# AI agents like Codex, Claude Code, or Goose might do:
# - Reading and analyzing source files
# - Creating new files (generated code, documentation, configs)
# - Modifying existing files (bug fixes, feature additions)
# - Generating comprehensive logs of all activities
class MockAgent
  attr_reader :workspace_path, :agent_id, :config, :logger

  # Configuration options for the mock agent
  DEFAULT_CONFIG = {
    # Work duration settings
    min_work_duration: 1.0,     # Minimum seconds to work
    max_work_duration: 10.0,    # Maximum seconds to work
    sleep_between_ops: 0.1,     # Sleep between file operations

    # File operation probabilities (0.0 - 1.0)
    read_file_probability: 0.8,
    create_file_probability: 0.6,
    modify_file_probability: 0.4,

    # Operation counts
    min_files_to_read: 2,
    max_files_to_read: 8,
    min_files_to_create: 1,
    max_files_to_create: 4,
    min_files_to_modify: 0,
    max_files_to_modify: 3,

    # Generated content settings
    generated_file_types: %w[.rb .py .js .md .txt .json .yml],
    max_generated_lines: 100,

    # Logging settings
    log_level: Logger::INFO,
    detailed_logging: true
  }.freeze

  def initialize(workspace_path, agent_id = nil, config = {})
    @workspace_path = File.expand_path(workspace_path)
    @agent_id = agent_id || "agent_#{Random.rand(10_000)}"
    @config = DEFAULT_CONFIG.merge(config)
    @activity_log = []

    validate_workspace
    setup_logger
  end

  # Main entry point - simulates a complete agent work session
  def run_work_session
    start_time = Time.now
    log_activity('AGENT_START', "Starting work session in #{@workspace_path}")

    begin
      # Determine work duration
      work_duration = random_duration
      log_activity('PLAN', "Planning to work for #{work_duration.round(2)} seconds")

      # Perform the work phases
      analyze_workspace
      perform_file_operations(work_duration)
      generate_summary_report

      success = true
    rescue StandardError => e
      log_activity('ERROR', "Work session failed: #{e.message}")
      @logger.error("Agent #{@agent_id} failed: #{e.message}")
      @logger.error(e.backtrace.join("\n"))
      success = false
    ensure
      end_time = Time.now
      duration = end_time - start_time
      log_activity('AGENT_END', "Work session completed in #{duration.round(2)} seconds")
    end

    # Return summary of work done
    {
      agent_id: @agent_id,
      workspace_path: @workspace_path,
      success: success,
      duration: duration,
      activity_count: @activity_log.size,
      activities: @activity_log
    }
  end

  # Get the current activity log
  def activity_log
    @activity_log.dup
  end

  private

  def setup_logger
    log_file = File.join(@workspace_path, '.agent_logs', "#{@agent_id}.log")
    FileUtils.mkdir_p(File.dirname(log_file))

    @logger = Logger.new(log_file)
    @logger.level = @config[:log_level]
    @logger.formatter = proc do |severity, datetime, _progname, msg|
      "[#{datetime.strftime('%Y-%m-%d %H:%M:%S')}] #{severity} [#{@agent_id}] #{msg}\n"
    end

    @logger.info("Mock agent #{@agent_id} initialized")
    @logger.info("Workspace: #{@workspace_path}")
    @logger.info("Config: #{@config.inspect}")
  end

  def validate_workspace
    raise ArgumentError, "Workspace path does not exist: #{@workspace_path}" unless Dir.exist?(@workspace_path)

    # Try to create a test file to ensure workspace is writable
    test_file = File.join(@workspace_path, ".agent_test_#{@agent_id}")
    File.write(test_file, 'test')
    File.delete(test_file)
  rescue StandardError => e
    raise ArgumentError, "Workspace is not writable: #{e.message}"
  end

  def log_activity(action, details)
    activity = {
      timestamp: Time.now.iso8601,
      agent_id: @agent_id,
      action: action,
      details: details
    }

    @activity_log << activity
    @logger.info("#{action}: #{details}") if @config[:detailed_logging]
  end

  def random_duration
    min = @config[:min_work_duration]
    max = @config[:max_work_duration]
    min + ((max - min) * Random.rand)
  end

  def analyze_workspace
    log_activity('ANALYZE_START', 'Beginning workspace analysis')

    # Find source files to analyze
    source_files = find_source_files
    log_activity('DISCOVER_FILES', "Found #{source_files.size} source files")

    # Simulate reading and analyzing files
    files_to_read = source_files.sample(random_count(:read))
    files_to_read.each do |file|
      read_and_analyze_file(file)
      sleep(@config[:sleep_between_ops])
    end

    log_activity('ANALYZE_END', "Analyzed #{files_to_read.size} files")
  end

  def perform_file_operations(target_duration)
    start_time = Time.now
    operations_performed = 0

    log_activity('OPERATIONS_START', 'Beginning file operations phase')

    while (Time.now - start_time) < target_duration
      operation = choose_operation
      perform_operation(operation)
      operations_performed += 1

      sleep(@config[:sleep_between_ops])

      # Occasionally check if we should continue
      break if Random.rand < 0.1 && (Time.now - start_time) > target_duration * 0.5
    end

    log_activity('OPERATIONS_END', "Performed #{operations_performed} operations")
  end

  def find_source_files
    extensions = %w[*.rb *.py *.js *.ts *.java *.cpp *.c *.h *.md *.txt *.json *.yml *.yaml]
    files = []

    extensions.each do |ext|
      files.concat(Dir.glob(File.join(@workspace_path, '**', ext)))
    end

    # Filter out generated files and logs
    files.reject! { |f| f.include?('/.agent_logs/') || f.include?('/generated_') }
    files
  end

  def read_and_analyze_file(file_path)
    content = File.read(file_path)
    relative_path = file_path.sub("#{@workspace_path}/", '')

    # Simulate analysis
    line_count = content.lines.size
    char_count = content.size
    hash = Digest::MD5.hexdigest(content)[0..7]

    log_activity('READ_FILE', "#{relative_path} (#{line_count} lines, #{char_count} chars, hash: #{hash})")

    # Simulate AI processing time based on file size
    processing_time = [0.1, content.size / 10_000.0].min
    sleep(processing_time)
  rescue StandardError => e
    log_activity('READ_ERROR', "Failed to read #{file_path}: #{e.message}")
  end

  def choose_operation
    operations = []
    operations << :create_file if Random.rand < @config[:create_file_probability]
    operations << :modify_file if Random.rand < @config[:modify_file_probability]
    operations << :read_file if Random.rand < @config[:read_file_probability]

    operations.empty? ? :read_file : operations.sample
  end

  def perform_operation(operation)
    case operation
    when :create_file
      create_generated_file
    when :modify_file
      modify_existing_file
    when :read_file
      file = find_source_files.sample
      read_and_analyze_file(file) if file
    end
  end

  def create_generated_file
    extension = @config[:generated_file_types].sample
    filename = "generated_#{@agent_id}_#{Random.rand(1000)}#{extension}"
    file_path = File.join(@workspace_path, filename)

    content = generate_file_content(extension)

    begin
      File.write(file_path, content)
      log_activity('CREATE_FILE', "Created #{filename} (#{content.lines.size} lines)")
    rescue StandardError => e
      log_activity('CREATE_ERROR', "Failed to create #{filename}: #{e.message}")
    end
  end

  def modify_existing_file
    source_files = find_source_files
    return if source_files.empty?

    file_path = source_files.sample
    relative_path = file_path.sub("#{@workspace_path}/", '')

    begin
      content = File.read(file_path)
      original_lines = content.lines.size

      # Simulate modification (add a comment)
      modification = "\n# Modified by #{@agent_id} at #{Time.now}\n"
      modified_content = content + modification

      File.write(file_path, modified_content)
      new_lines = modified_content.lines.size

      log_activity('MODIFY_FILE', "Modified #{relative_path} (#{original_lines} -> #{new_lines} lines)")
    rescue StandardError => e
      log_activity('MODIFY_ERROR', "Failed to modify #{relative_path}: #{e.message}")
    end
  end

  def generate_file_content(extension)
    lines = Random.rand(@config[:max_generated_lines]) + 5

    case extension
    when '.rb'
      generate_ruby_content(lines)
    when '.py'
      generate_python_content(lines)
    when '.js'
      generate_javascript_content(lines)
    when '.md'
      generate_markdown_content(lines)
    when '.json'
      generate_json_content
    when '.yml', '.yaml'
      generate_yaml_content
    else
      generate_generic_content(lines)
    end
  end

  def generate_ruby_content(lines)
    content = "# Generated by MockAgent #{@agent_id}\n"
    content += "# Created at #{Time.now}\n\n"
    content += "class Generated#{@agent_id.capitalize}\n"

    (lines - 10).times do |i|
      content += "  # Method #{i + 1}\n"
      content += "  def method_#{i + 1}\n"
      content += "    puts 'Hello from method #{i + 1}'\n"
      content += "  end\n\n"
    end

    content += "end\n"
    content
  end

  def generate_python_content(lines)
    content = "# Generated by MockAgent #{@agent_id}\n"
    content += "# Created at #{Time.now}\n\n"
    content += "class Generated#{@agent_id.capitalize}:\n"

    (lines - 8).times do |i|
      content += "    def method_#{i + 1}(self):\n"
      content += "        print(f'Hello from method #{i + 1}')\n\n"
    end

    content
  end

  def generate_javascript_content(lines)
    content = "// Generated by MockAgent #{@agent_id}\n"
    content += "// Created at #{Time.now}\n\n"
    content += "class Generated#{@agent_id.capitalize} {\n"

    (lines - 8).times do |i|
      content += "  method#{i + 1}() {\n"
      content += "    console.log('Hello from method #{i + 1}');\n"
      content += "  }\n\n"
    end

    content += "}\n"
    content
  end

  def generate_markdown_content(lines)
    content = "# Generated Documentation\n\n"
    content += "Generated by MockAgent #{@agent_id} at #{Time.now}\n\n"

    (lines - 10).times do |i|
      content += "## Section #{i + 1}\n\n"
      content += "This is section #{i + 1} of the generated documentation.\n\n"
      content += "- Item 1\n- Item 2\n- Item 3\n\n"
    end

    content
  end

  def generate_json_content
    SimpleJSON.generate({
                          generator: 'MockAgent',
                          agent_id: @agent_id,
                          created_at: Time.now.iso8601,
                          data: {
                            items: (1..Random.rand(1..10)).map do |i|
                              { id: i, name: "Item #{i}", value: Random.rand(100) }
                            end
                          }
                        })
  end

  def generate_yaml_content
    "# Generated by MockAgent #{@agent_id}\n" \
    "metadata:\n  " \
    "agent_id: #{@agent_id}\n  " \
    "created_at: #{Time.now.iso8601}\n" \
    "config:\n  " \
    "setting1: value1\n  " \
    "setting2: value2\n  " \
    "items:\n" +
      (1..5).map { |i| "    - name: Item #{i}\n      value: #{Random.rand(100)}" }.join("\n")
  end

  def generate_generic_content(lines)
    content = "Generated by MockAgent #{@agent_id}\n"
    content += "Created at #{Time.now}\n\n"

    lines.times do |i|
      content += "Line #{i + 1}: This is generated content line #{i + 1}\n"
    end

    content
  end

  def generate_summary_report
    log_file = File.join(@workspace_path, '.agent_logs', "#{@agent_id}_summary.json")

    summary = {
      agent_id: @agent_id,
      workspace_path: @workspace_path,
      session_start: @activity_log.first&.dig(:timestamp),
      session_end: @activity_log.last&.dig(:timestamp),
      total_activities: @activity_log.size,
      activities_by_type: @activity_log.group_by { |a| a[:action] }.transform_values(&:size),
      workspace_files_created: Dir.glob(File.join(@workspace_path, "generated_#{@agent_id}_*")).size
    }

    File.write(log_file, SimpleJSON.pretty_generate(summary))
    log_activity('SUMMARY_REPORT', "Generated summary report at #{File.basename(log_file)}")
  end

  def random_count(type)
    min_key = :"min_files_to_#{type}"
    max_key = :"max_files_to_#{type}"
    min = @config[min_key] || 1
    max = @config[max_key] || 3
    Random.rand(max - min + 1) + min
  end
end
