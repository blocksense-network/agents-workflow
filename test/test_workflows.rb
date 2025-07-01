# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative 'test_helper'

class WorkflowTest < Minitest::Test
  include RepoTestHelper

  def prepare_workflows(dir)
    flow_dir = File.join(dir, '.agents', 'workflows')
    FileUtils.mkdir_p(flow_dir)
    File.write(File.join(flow_dir, 'sh'), "#!/bin/bash\necho shell\necho '@agents-setup SH=1'\n")
    File.chmod(0o644, File.join(flow_dir, 'sh'))
    File.write(File.join(flow_dir, 'rb'), "#!/usr/bin/env ruby\nputs 'ruby'\nputs '@agents-setup RB=2'\n")
    File.chmod(0o755, File.join(flow_dir, 'rb'))
    File.write(File.join(flow_dir, 'msg.txt'), "text line\n")
  end

  def test_workflow_commands
    RepoTestHelper::AGENT_TASK_BINARIES.product(RepoTestHelper::GET_TASK_BINARIES).each do |ab, gb|
      repo, remote = setup_repo(:git)
      prepare_workflows(repo)
      status, = run_agent_task(repo, branch: 'feat', lines: ['start', '/sh', '/rb', '/msg'], push_to_remote: false,
                                     tool: ab)
      # agent-task should succeed with workflows
      assert_equal 0, status.exitstatus
      VCSRepo.new(repo).checkout_branch('feat')
      status2, output = run_get_task(repo, tool: gb)
      # outputs of workflows should appear
      assert_equal 0, status2.exitstatus
      assert_includes output, 'shell'
      assert_includes output, 'ruby'
      assert_includes output, 'text line'
      refute_includes output, '@agents-setup'
      status3, env_output = run_get_task(repo, tool: gb, args: ['--get-setup-env'])
      assert_equal 0, status3.exitstatus
      assert_includes env_output, 'SH=1'
      assert_includes env_output, 'RB=2'
    ensure
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_missing_workflow_errors
    RepoTestHelper::AGENT_TASK_BINARIES.each do |ab|
      repo, remote = setup_repo(:git)
      status, output = run_agent_task(repo, branch: 'bad', lines: ['/missing'], push_to_remote: false, tool: ab)
      # missing workflow should cause failure
      assert output.include?('Unknown workflow command')
      refute_equal 0, status.exitstatus
    ensure
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_prompt_option_validation
    RepoTestHelper::AGENT_TASK_BINARIES.each do |ab|
      repo, remote = setup_repo(:git)
      status, output, = run_agent_task(repo, branch: 'p1', prompt: '/missing', push_to_remote: false, tool: ab)
      # prompt mode should exit with error
      assert output.include?('Unknown workflow command')
      refute_equal 0, status.exitstatus
    ensure
      FileUtils.remove_entry(repo) if repo && File.exist?(repo)
      FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    end
  end

  def test_setup_env_propagation
    repo, remote = setup_repo(:git)
    flow_dir = File.join(repo, '.agents', 'workflows')
    FileUtils.mkdir_p(flow_dir)
    File.write(File.join(flow_dir, 'env.txt'), "@agents-setup TESTVAR=42\n")
    FileUtils.mkdir_p(File.join(repo, '.agents'))
    File.write(File.join(repo, '.agents', 'codex-setup'), "#!/bin/bash\necho $TESTVAR > result.txt\n")
    File.chmod(0o755, File.join(repo, '.agents', 'codex-setup'))
    status, = run_agent_task(repo, branch: 'feat', lines: ['/env'], push_to_remote: false, tool: RepoTestHelper::AGENT_TASK)
    assert_equal 0, status.exitstatus
    VCSRepo.new(repo).checkout_branch('feat')
    # clone workflow repo so that codex-setup moves itself safely
    tmp = Dir.mktmpdir('clone')
    FileUtils.cp_r(ROOT, File.join(tmp, 'agents-workflow'))
    bin_dir = Dir.mktmpdir('bin')
    File.write(File.join(bin_dir, 'sudo'), "#!/bin/sh\nexec \"$@\"\n")
    File.chmod(0o755, File.join(bin_dir, 'sudo'))
    env = { 'PATH' => "#{bin_dir}:#{ENV.fetch('PATH', nil)}" }
    system(env, File.join(tmp, 'agents-workflow', 'codex-setup'), chdir: repo)
    result = File.read(File.join(repo, 'result.txt'))
    assert_equal "42\n", result
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    FileUtils.remove_entry(tmp) if defined?(tmp) && File.exist?(tmp)
    FileUtils.remove_entry(bin_dir) if defined?(bin_dir) && File.exist?(bin_dir)
  end
end
