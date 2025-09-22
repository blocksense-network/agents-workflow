# frozen_string_literal: true

require_relative 'test_helper'
require 'mock_agent'
require 'tmpdir'
require 'json'

# Unit tests for MockAgent behavior and functionality
class TestMockAgent < Minitest::Test
  def setup
    @test_workspace = Dir.mktmpdir('mock_agent_test')
    create_test_files
  end

  def teardown
    FileUtils.rm_rf(@test_workspace)
  end

  def test_agent_initialization
    agent = MockAgent.new(@test_workspace, 'test_agent')

    assert_equal @test_workspace, agent.workspace_path
    assert_equal 'test_agent', agent.agent_id
    assert_respond_to agent, :run_work_session
    assert_respond_to agent, :activity_log
  end

  def test_agent_initialization_with_config
    config = {
      min_work_duration: 0.1,
      max_work_duration: 0.5,
      create_file_probability: 1.0,
      log_level: Logger::DEBUG
    }

    agent = MockAgent.new(@test_workspace, 'configured_agent', config)

    assert_equal 0.1, agent.config[:min_work_duration]
    assert_equal 0.5, agent.config[:max_work_duration]
    assert_equal 1.0, agent.config[:create_file_probability]
    assert_equal Logger::DEBUG, agent.config[:log_level]
  end

  def test_agent_auto_generated_id
    agent = MockAgent.new(@test_workspace)

    assert_match(/^agent_\d+$/, agent.agent_id)
    refute_equal agent.agent_id, MockAgent.new(@test_workspace).agent_id
  end

  def test_invalid_workspace_path
    nonexistent_path = File.join(Dir.tmpdir, "definitely_nonexistent_#{rand(1_000_000)}")
    assert_raises(ArgumentError) do
      MockAgent.new(nonexistent_path)
    end
  end

  def test_readonly_workspace
    # Make workspace read-only
    File.chmod(0o444, @test_workspace)

    assert_raises(ArgumentError) do
      MockAgent.new(@test_workspace)
    end
  ensure
    # Restore permissions for cleanup
    File.chmod(0o755, @test_workspace)
  end

  def test_work_session_execution
    agent = MockAgent.new(@test_workspace, 'session_test', {
                            min_work_duration: 0.1,
                            max_work_duration: 0.3,
                            create_file_probability: 1.0,
                            modify_file_probability: 0.5
                          })

    result = agent.run_work_session

    assert result[:success], 'Work session should succeed'
    assert_equal 'session_test', result[:agent_id]
    assert_equal @test_workspace, result[:workspace_path]
    assert result[:duration] > 0.05, 'Session should take some time'
    assert result[:activity_count].positive?, 'Session should log activities'
    assert_instance_of Array, result[:activities]
  end

  def test_file_creation_behavior
    agent = MockAgent.new(@test_workspace, 'creator', {
                            min_work_duration: 0.1,
                            max_work_duration: 0.2,
                            create_file_probability: 1.0,
                            modify_file_probability: 0.0,
                            read_file_probability: 0.0
                          })

    result = agent.run_work_session

    # Check that files were created
    generated_files = Dir.glob(File.join(@test_workspace, 'generated_creator_*'))
    assert generated_files.size.positive?, 'Agent should create at least one file'

    # Verify files have content
    generated_files.each do |file|
      content = File.read(file)
      assert content.size.positive?, 'Generated file should have content'
      assert content.include?('creator'), 'Generated file should include agent ID'
    end

    # Check activity log
    create_activities = result[:activities].select { |a| a[:action] == 'CREATE_FILE' }
    assert create_activities.size.positive?, 'Should log file creation activities'
  end

  def test_file_modification_behavior
    agent = MockAgent.new(@test_workspace, 'modifier', {
                            min_work_duration: 1.0,
                            max_work_duration: 1.5,
                            create_file_probability: 0.0,
                            modify_file_probability: 1.0,
                            read_file_probability: 0.0
                          })

    # Get original content of test files
    original_contents = {}
    Dir.glob(File.join(@test_workspace, '*.txt')).each do |file|
      original_contents[file] = File.read(file)
    end

    result = agent.run_work_session

    # Check for modifications
    modify_activities = result[:activities].select { |a| a[:action] == 'MODIFY_FILE' }
    return unless modify_activities.size.positive?

    # At least one file should be modified
    modified = false
    original_contents.each do |file, original_content|
      next unless File.exist?(file)

      new_content = File.read(file)
      if new_content != original_content
        modified = true
        assert new_content.include?('modifier'), 'Modified file should include agent ID'
      end
    end

    assert modified, 'At least one file should be modified' if modify_activities.size.positive?
  end

  def test_file_reading_behavior
    agent = MockAgent.new(@test_workspace, 'reader', {
                            min_work_duration: 0.1,
                            max_work_duration: 0.2,
                            create_file_probability: 0.0,
                            modify_file_probability: 0.0,
                            read_file_probability: 1.0
                          })

    result = agent.run_work_session

    # Check activity log for file reads
    read_activities = result[:activities].select { |a| a[:action] == 'READ_FILE' }
    assert read_activities.size.positive?, 'Agent should read files'

    # Verify read activities have meaningful details
    read_activities.each do |activity|
      assert activity[:details].match(/\(\d+ lines, \d+ chars/), 'Read activity should include file stats'
    end
  end

  def test_logging_functionality
    agent = MockAgent.new(@test_workspace, 'logger_test', {
                            min_work_duration: 0.1,
                            max_work_duration: 0.2,
                            detailed_logging: true
                          })

    agent.run_work_session

    # Check that log file was created
    log_file = File.join(@test_workspace, '.agent_logs', 'logger_test.log')
    assert File.exist?(log_file), 'Agent should create log file'

    # Check log content
    log_content = File.read(log_file)
    assert log_content.include?('logger_test'), 'Log should include agent ID'
    assert log_content.include?('AGENT_START'), 'Log should include session start'
    assert log_content.include?('AGENT_END'), 'Log should include session end'

    # Check summary report
    summary_file = File.join(@test_workspace, '.agent_logs', 'logger_test_summary.json')
    assert File.exist?(summary_file), 'Agent should create summary report'

    summary_content = File.read(summary_file)
    # Just verify that the summary file contains expected content
    assert summary_content.include?('logger_test'), 'Summary should include agent ID'
    assert summary_content.include?('activities_by_type'), 'Summary should include activities breakdown'
  end

  def test_activity_log_structure
    agent = MockAgent.new(@test_workspace, 'activity_test', {
                            min_work_duration: 0.1,
                            max_work_duration: 0.2
                          })

    result = agent.run_work_session

    # Verify activity log structure
    assert_instance_of Array, result[:activities]
    assert result[:activities].size.positive?, 'Should have activities'

    result[:activities].each do |activity|
      assert_instance_of Hash, activity
      assert activity.key?(:timestamp), 'Activity should have timestamp'
      assert activity.key?(:agent_id), 'Activity should have agent_id'
      assert activity.key?(:action), 'Activity should have action'
      assert activity.key?(:details), 'Activity should have details'

      assert_equal 'activity_test', activity[:agent_id]
      assert_instance_of String, activity[:action]
      assert_instance_of String, activity[:details]
    end

    # Check for expected activity types
    actions = result[:activities].map { |a| a[:action] }
    assert actions.include?('AGENT_START'), 'Should include AGENT_START'
    assert actions.include?('AGENT_END'), 'Should include AGENT_END'
    assert actions.include?('ANALYZE_START'), 'Should include ANALYZE_START'
  end

  def test_concurrent_agents_different_ids
    threads = []

    # Create multiple agents with auto-generated IDs
    3.times do
      thread = Thread.new do
        agent = MockAgent.new(@test_workspace, nil, {
                                min_work_duration: 0.1,
                                max_work_duration: 0.2
                              })
        Thread.current[:agent_id] = agent.agent_id
        agent.run_work_session
      end
      threads << thread
    end

    # Get all agent IDs
    agent_ids = threads.map do |thread|
      thread.join
      thread[:agent_id]
    end

    # Verify all IDs are unique
    assert_equal agent_ids.size, agent_ids.uniq.size, 'All agent IDs should be unique'
  end

  def test_different_file_type_generation
    file_types = %w[.rb .py .js .md .json .yml]

    file_types.each do |ext|
      agent = MockAgent.new(@test_workspace, "gen_#{ext.gsub('.', '')}", {
                              min_work_duration: 1.0,
                              max_work_duration: 1.5,
                              create_file_probability: 1.0,
                              generated_file_types: [ext],
                              max_generated_lines: 10
                            })

      agent.run_work_session

      # Check that a file with the expected extension was created
      all_files_with_ext = Dir.glob(File.join(@test_workspace, "*#{ext}"))
      generated_files = all_files_with_ext.select { |f| File.basename(f).include?('generated') }
      assert generated_files.size.positive?, "Should generate #{ext} file"

      # Check file content is appropriate for type
      generated_files.each do |file|
        content = File.read(file)
        case ext
        when '.rb'
          assert content.include?('class') || content.include?('def'), 'Ruby file should have Ruby syntax'
        when '.py'
          assert content.include?('class') || content.include?('def'), 'Python file should have Python syntax'
        when '.js'
          assert content.include?('class') || content.include?('function'), 'JS file should have JS syntax'
        when '.json'
          # Just check that the content looks like JSON-ish structure
          assert content.include?('{') && content.include?('}'), 'JSON file should have JSON-like structure'
        end
      end
    end
  end

  private

  def create_test_files
    # Create some test files for agents to work with
    File.write(File.join(@test_workspace, 'test1.txt'), "This is test file 1\nWith multiple lines\n")
    File.write(File.join(@test_workspace, 'test2.txt'), "This is test file 2\nAlso with content\n")
    File.write(File.join(@test_workspace, 'sample.rb'), "puts 'Hello, World!'\n")

    # Create subdirectory with files
    FileUtils.mkdir_p(File.join(@test_workspace, 'subdir'))
    File.write(File.join(@test_workspace, 'subdir', 'nested.txt'), "Nested file content\n")
  end
end
