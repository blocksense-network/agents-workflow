# frozen_string_literal: true

require_relative 'test_helper'
require 'snapshot/provider'
require 'tmpdir'
require 'open3'

# Integration tests using Docker containers to test MockAgent in isolated environments
# These tests verify that the Docker-based testing environment works correctly
class TestMockAgentDocker < Minitest::Test
  def setup
    @test_repo = Dir.mktmpdir('docker_test_repo')
    @workspaces = []

    # Skip if Docker is not available
    skip_unless_docker_available

    create_test_repository
    build_docker_image
  end

  def teardown
    cleanup_workspaces
    FileUtils.rm_rf(@test_repo)
  end

  def test_docker_image_builds_successfully
    # The build_docker_image method in setup should have succeeded
    # if we get here without skipping
    assert image_exists?, 'Docker image should be built successfully'
  end

  def test_single_agent_in_docker_container
    workspace = create_docker_workspace('single_docker')

    # Run MockAgent in Docker container
    cmd = build_docker_run_command(workspace, 'docker_test_1', {
                                     '--min-duration' => '0.5',
                                     '--max-duration' => '2.0',
                                     '--create-probability' => '1.0'
                                   })

    stdout, stderr, status = Open3.capture3(*cmd)

    assert status.success?, "Docker container should run successfully\nSTDOUT: #{stdout}\nSTDERR: #{stderr}"
    assert stdout.include?('MockAgent docker_test_1 completed'), 'Should show completion message'
    assert stdout.include?('Success: true'), 'Agent should complete successfully'

    # Verify files were created in the workspace
    generated_files = Dir.glob(File.join(workspace, 'generated_docker_test_1_*'))
    assert generated_files.size.positive?, 'Agent should create files in mounted workspace'

    # Verify log files exist
    log_dir = File.join(workspace, '.agent_logs')
    assert Dir.exist?(log_dir), 'Agent should create log directory'

    log_files = Dir.glob(File.join(log_dir, '*'))
    assert log_files.size >= 2, 'Agent should create log and summary files'
  end

  def test_concurrent_docker_containers
    num_containers = 3
    workspaces = []
    commands = []

    # Create workspaces and commands for concurrent execution
    (1..num_containers).each do |i|
      workspace = create_docker_workspace("concurrent_#{i}")
      workspaces << workspace

      cmd = build_docker_run_command(workspace, "concurrent_#{i}", {
                                       '--min-duration' => '1.0',
                                       '--max-duration' => '2.0',
                                       '--create-probability' => '0.8',
                                       '--modify-probability' => '0.5'
                                     })

      commands << cmd
    end

    # Execute containers concurrently
    start_time = Time.now
    threads = commands.map.with_index do |cmd, i|
      Thread.new do
        stdout, stderr, status = Open3.capture3(*cmd)
        {
          index: i,
          stdout: stdout,
          stderr: stderr,
          status: status,
          workspace: workspaces[i]
        }
      end
    end

    results = threads.map(&:join).map(&:value)
    total_time = Time.now - start_time

    # Verify all containers completed successfully
    results.each do |result|
      assert result[:status].success?,
             "Container #{result[:index]} should succeed\nSTDOUT: #{result[:stdout]}\nSTDERR: #{result[:stderr]}"
      assert result[:stdout].include?('Success: true'), 'Agent should complete successfully'
    end

    # Verify isolation - each workspace should have unique files
    workspaces.each_with_index do |workspace, i|
      agent_id = "concurrent_#{i + 1}"

      # Files created by this agent
      agent_files = Dir.glob(File.join(workspace, "generated_#{agent_id}_*"))
      assert agent_files.size.positive?, "Agent #{i + 1} should create files"

      # Files created by other agents (should be none)
      other_agent_files = (1..num_containers).reject { |j| j == i + 1 }.flat_map do |j|
        other_agent_id = "concurrent_#{j}"
        Dir.glob(File.join(workspace, "generated_#{other_agent_id}_*"))
      end

      assert_empty other_agent_files, "Workspace #{i + 1} should not contain other agents' files"
    end

    puts "\nDocker concurrent execution summary:"
    puts "  Containers: #{num_containers}"
    puts "  Total time: #{total_time.round(2)}s"
    puts "  All successful: #{results.all? { |r| r[:status].success? }}"
  end

  def test_docker_container_with_different_configurations
    configurations = [
      {
        name: 'fast_worker',
        args: {
          '--min-duration' => '0.2',
          '--max-duration' => '0.5',
          '--create-probability' => '0.3',
          '--sleep-between' => '0.01'
        }
      },
      {
        name: 'heavy_creator',
        args: {
          '--min-duration' => '1.0',
          '--max-duration' => '1.5',
          '--create-probability' => '1.0',
          '--modify-probability' => '0.8'
        }
      },
      {
        name: 'reader_analyzer',
        args: {
          '--min-duration' => '0.8',
          '--max-duration' => '1.2',
          '--read-probability' => '1.0',
          '--create-probability' => '0.2',
          '--verbose' => nil # flag without value
        }
      }
    ]

    configurations.each do |config|
      workspace = create_docker_workspace(config[:name])

      cmd = build_docker_run_command(workspace, config[:name], config[:args])
      stdout, stderr, status = Open3.capture3(*cmd)

      assert status.success?,
             "Container #{config[:name]} should succeed\nSTDOUT: #{stdout}\nSTDERR: #{stderr}"
      assert stdout.include?('Success: true'), "Agent #{config[:name]} should complete"

      # Verify behavior matches configuration
      case config[:name]
      when 'heavy_creator'
        generated_files = Dir.glob(File.join(workspace, "generated_#{config[:name]}_*"))
        assert generated_files.size >= 2, 'Heavy creator should create multiple files'

      when 'reader_analyzer'
        assert stdout.include?('Configuration:'), 'Verbose mode should show configuration'

      when 'fast_worker'
        # Fast worker should complete quickly, but this is hard to verify reliably
        # in a test environment
      end
    end
  end

  def test_docker_container_error_handling
    workspace = create_docker_workspace('error_test')

    # Create a Docker command with invalid arguments
    cmd = [
      'docker', 'run', '--rm',
      '-v', "#{workspace}:/workspace",
      'mock-agent-test:latest',
      '--invalid-option'
    ]

    stdout, stderr, status = Open3.capture3(*cmd)

    refute status.success?, 'Container should fail with invalid arguments'
    assert stderr.include?('invalid option') || stdout.include?('Error:'),
           'Should show error message for invalid option'
  end

  def test_docker_workspace_persistence
    workspace = create_docker_workspace('persistence_test')

    # Run first agent
    cmd1 = build_docker_run_command(workspace, 'persist_1', {
                                      '--min-duration' => '0.5',
                                      '--max-duration' => '1.0',
                                      '--create-probability' => '1.0'
                                    })

    _, _, status1 = Open3.capture3(*cmd1)
    assert status1.success?, 'First container should succeed'

    # Check files created by first agent
    files_after_first = Dir.glob(File.join(workspace, 'generated_persist_1_*'))
    assert files_after_first.size.positive?, 'First agent should create files'

    # Run second agent in same workspace
    cmd2 = build_docker_run_command(workspace, 'persist_2', {
                                      '--min-duration' => '0.5',
                                      '--max-duration' => '1.0',
                                      '--create-probability' => '1.0',
                                      '--modify-probability' => '0.5'
                                    })

    _, _, status2 = Open3.capture3(*cmd2)
    assert status2.success?, 'Second container should succeed'

    # Check that both agents' files exist
    files_from_first = Dir.glob(File.join(workspace, 'generated_persist_1_*'))
    files_from_second = Dir.glob(File.join(workspace, 'generated_persist_2_*'))

    assert files_from_first.size.positive?, "First agent's files should persist"
    assert files_from_second.size.positive?, 'Second agent should create new files'

    # Verify log files from both agents exist
    log_files = Dir.glob(File.join(workspace, '.agent_logs', '*'))
    assert log_files.any? { |f| f.include?('persist_1') }, "First agent's logs should exist"
    assert log_files.any? { |f| f.include?('persist_2') }, "Second agent's logs should exist"
  end

  private

  def skip_unless_docker_available
    _, _, status = Open3.capture3('docker', '--version')
    return if status.success?

    skip 'Docker is not available on this system'
  end

  def image_exists?
    stdout, _, status = Open3.capture3('docker', 'images', '-q', 'mock-agent-test:latest')
    status.success? && !stdout.strip.empty?
  end

  def build_docker_image
    dockerfile_path = File.join(File.dirname(__FILE__), 'Dockerfile.mock-agent')
    context_path = File.expand_path('../..', File.dirname(__FILE__))

    cmd = [
      'docker', 'build',
      '-t', 'mock-agent-test:latest',
      '-f', dockerfile_path,
      context_path
    ]

    stdout, stderr, status = Open3.capture3(*cmd)

    return if status.success?

    puts 'Failed to build Docker image:'
    puts "STDOUT: #{stdout}"
    puts "STDERR: #{stderr}"
    skip 'Could not build Docker image for testing'
  end

  def create_test_repository
    # Create test repository structure
    FileUtils.mkdir_p(File.join(@test_repo, 'src'))
    FileUtils.mkdir_p(File.join(@test_repo, 'test'))
    FileUtils.mkdir_p(File.join(@test_repo, 'docs'))

    File.write(File.join(@test_repo, 'README.md'), <<~README)
      # Docker Test Repository

      This repository is used for testing MockAgent in Docker containers.

      ## Files

      - Source code in `src/`
      - Tests in `test/`
      - Documentation in `docs/`
    README

    File.write(File.join(@test_repo, 'src', 'app.rb'), <<~RUBY)
      #!/usr/bin/env ruby

      class App
        def initialize
          @name = "Docker Test App"
        end

        def run
          puts "Running \#{@name}"
        end
      end

      if __FILE__ == $0
        App.new.run
      end
    RUBY

    File.write(File.join(@test_repo, 'test', 'test_app.rb'), <<~RUBY)
      require 'minitest/autorun'
      require_relative '../src/app'

      class TestApp < Minitest::Test
        def test_app_creation
          app = App.new
          assert_instance_of App, app
        end
      end
    RUBY

    File.write(File.join(@test_repo, 'docs', 'usage.md'), <<~MARKDOWN)
      # Usage Guide

      ## Running the Application

      ```bash
      ruby src/app.rb
      ```

      ## Running Tests

      ```bash
      ruby test/test_app.rb
      ```
    MARKDOWN
  end

  def create_docker_workspace(name)
    # Use snapshot provider to create isolated workspace
    provider = Snapshot.provider_for(@test_repo)
    workspace = Dir.mktmpdir("docker_workspace_#{name}")
    @workspaces << workspace

    provider.create_workspace(workspace)
    workspace
  end

  def build_docker_run_command(workspace, agent_id, args = {})
    cmd = [
      'docker', 'run', '--rm',
      '-v', "#{workspace}:/workspace",
      'mock-agent-test:latest',
      '--agent-id', agent_id
    ]

    # Add configuration arguments
    args.each do |key, value|
      cmd << key
      cmd << value.to_s if value # Some flags don't have values
    end

    cmd
  end

  def cleanup_workspaces
    @workspaces.each do |workspace|
      provider = Snapshot.provider_for(@test_repo)
      provider.cleanup_workspace(workspace) if provider.respond_to?(:cleanup_workspace)
      FileUtils.rm_rf(workspace)
    rescue StandardError => e
      puts "Warning: Failed to cleanup workspace #{workspace}: #{e.message}"
    end
    @workspaces.clear
  end
end
