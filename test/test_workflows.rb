# frozen_string_literal: true

require 'minitest/autorun'
require 'tmpdir'
require 'fileutils'
require_relative 'test_helper'

class WorkflowTest < Minitest::Test
  include RepoTestHelper

  def test_workflow_expansion_and_env
    repo, remote = setup_repo(:git)
    wf_dir = File.join(repo, '.agents', 'workflows')
    FileUtils.mkdir_p(wf_dir)
    hello = File.join(wf_dir, 'hello')
    File.write(hello, "#!/bin/sh\necho hello\necho '@agents-setup FOO=bar'")
    FileUtils.chmod(0o755, hello)
    bye = File.join(wf_dir, 'bye')
    File.write(bye, "#!/bin/sh\necho bye")
    FileUtils.chmod(0o755, bye)

    prompt = "/hello   \nThis task uses two workflows.\n/bye   \n@agents-setup BAZ=1"
    status, = run_agent_task(repo, branch: 'feat', prompt: prompt, push_to_remote: false)
    # agent-task should succeed when workflows run
    assert_equal 0, status.exitstatus

    VCSRepo.new(repo).checkout_branch('feat')
    _, output = run_get_task(repo)
    # output should include results from both workflows
    assert_includes output, 'hello'
    assert_includes output, 'bye'
    # setup directives should be stripped from the output
    refute_includes output, '@agents-setup'

    cmd = if windows?
            ['ruby', RepoTestHelper::GET_TASK, '--get-setup-env']
          else
            [RepoTestHelper::GET_TASK, '--get-setup-env']
          end

    env_output = IO.popen(cmd, chdir: repo, &:read)
    status2 = $CHILD_STATUS
    # `get-task --get-setup-env` should succeed
    assert_equal 0, status2.exitstatus
    lines = env_output.split("\n")
    # env vars from workflow and task should be present
    assert_includes lines, 'FOO=bar'
    assert_includes lines, 'BAZ=1'
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end
end

class WorkflowAdditionalTest < Minitest::Test
  include RepoTestHelper

  def create_workflow(repo, name, content, mode: 0o755)
    dir = File.join(repo, '.agents', 'workflows')
    FileUtils.mkdir_p(dir)
    path = File.join(dir, name)
    File.write(path, content)
    File.chmod(mode, path)
    path
  end

  def test_ruby_workflow_command
    repo, remote = setup_repo(:git)
    create_workflow(repo, 'ruby_wf', <<~RUBY)
      #!/usr/bin/env ruby
      puts 'ruby works'
      puts '@agents-setup RUBY_FLAG=1'
    RUBY

    prompt = "This task demonstrates a ruby workflow.\n/ruby_wf   "
    status, = run_agent_task(repo, branch: 'feat', prompt: prompt, push_to_remote: false)
    # agent-task should succeed with ruby workflow
    assert_equal 0, status.exitstatus

    VCSRepo.new(repo).checkout_branch('feat')
    _, output = run_get_task(repo)
    # get-task should include workflow output
    assert_includes output, 'ruby works'

    cmd = if windows?
            ['ruby', RepoTestHelper::GET_TASK, '--get-setup-env']
          else
            [RepoTestHelper::GET_TASK, '--get-setup-env']
          end
    env_out = IO.popen(cmd, chdir: repo, &:read)
    status2 = $CHILD_STATUS
    # env listing should succeed
    assert_equal 0, status2.exitstatus
    # ruby workflow env vars should be present
    assert_includes env_out.split("\n"), 'RUBY_FLAG=1'
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_text_workflow_command
    repo, remote = setup_repo(:git)
    dir = File.join(repo, '.agents', 'workflows')
    FileUtils.mkdir_p(dir)
    File.write(File.join(dir, 'info.txt'), 'hello from txt')

    prompt = "/info\nSome additional details about the task."
    status, = run_agent_task(repo, branch: 'feat', prompt: prompt, push_to_remote: false)
    # agent-task should succeed with txt workflow
    assert_equal 0, status.exitstatus

    VCSRepo.new(repo).checkout_branch('feat')
    _, output = run_get_task(repo)
    # get-task should include contents of the txt file
    assert_includes output, 'hello from txt'
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_workflow_with_arguments
    repo, remote = setup_repo(:git)
    create_workflow(repo, 'echo_args', "#!/bin/sh\necho $1 $2")

    prompt = "Before running commands.\n/echo_args foo \"bar baz\"\n/echo_args qux quux   \nAfter commands."
    status, = run_agent_task(repo, branch: 'feat', prompt: prompt, push_to_remote: false)
    # agent-task should succeed when passing arguments twice
    assert_equal 0, status.exitstatus

    VCSRepo.new(repo).checkout_branch('feat')
    _, output = run_get_task(repo)
    # workflow should receive correctly parsed arguments
    assert_includes output, 'foo bar baz'
    # second invocation should also appear in output
    assert_includes output, 'qux quux'
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_setup_script_receives_env_vars
    repo, remote = setup_repo(:git)
    create_workflow(repo, 'envgen', "#!/bin/sh\necho '@agents-setup FOO=BAR'")

    agents_dir = File.join(repo, '.agents')
    FileUtils.mkdir_p(agents_dir)
    result_file = File.join(repo, 'setup_result')
    File.write(File.join(agents_dir, 'codex-setup'), "#!/bin/sh\necho $FOO > #{result_file}\n")
    File.chmod(0o755, File.join(agents_dir, 'codex-setup'))

    prompt = "Prepare env.\n/envgen\nDone."
    status, = run_agent_task(repo, branch: 'feat', prompt: prompt, push_to_remote: false)
    # agent-task should succeed before running setup
    assert_equal 0, status.exitstatus

    VCSRepo.new(repo).checkout_branch('feat')

    clone = Dir.mktmpdir('aw_clone')
    FileUtils.cp_r(File.join(RepoTestHelper::ROOT, '.'), clone)

    stub_dir = Dir.mktmpdir('stubs')
    File.write(File.join(stub_dir, 'sudo'), "#!/bin/sh\nexit 0\n")
    File.chmod(0o755, File.join(stub_dir, 'sudo'))
    File.write(File.join(stub_dir, 'mv'), "#!/bin/sh\nexit 0\n")
    File.chmod(0o755, File.join(stub_dir, 'mv'))

    path_env = ENV.fetch('PATH', nil)
    env = { 'PATH' => "#{stub_dir}:#{path_env}", 'HOME' => Dir.mktmpdir('home') }
    IO.popen(env, [File.join(clone, 'codex-setup')], chdir: repo, &:read)

    # setup script from repo should capture env variable
    assert_equal 'BAR', File.read(result_file).strip
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
    FileUtils.remove_entry(clone) if defined?(clone) && clone && File.exist?(clone)
    FileUtils.remove_entry(stub_dir) if defined?(stub_dir) && stub_dir && File.exist?(stub_dir)
    FileUtils.rm_rf('/tmp/agents-workflow')
  end
end

class WorkflowDiagnosticsTest < Minitest::Test
  include RepoTestHelper

  def create_workflow(repo, name, content)
    dir = File.join(repo, '.agents', 'workflows')
    FileUtils.mkdir_p(dir)
    path = File.join(dir, name)
    File.write(path, content)
    File.chmod(0o755, path)
    path
  end

  def test_unknown_workflow_command
    repo, remote = setup_repo(:git)
    at = AgentTasks.new(repo)
    _, _, diagnostics = at.process_workflows("/missing\nTrailing text")
    # should report missing workflow command
    assert_includes diagnostics, "Unknown workflow command '/missing'"
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_conflicting_env_assignments
    repo, remote = setup_repo(:git)
    create_workflow(repo, 'a', "#!/bin/sh\necho '@agents-setup VAR=1'")
    create_workflow(repo, 'b', "#!/bin/sh\necho '@agents-setup VAR=2'")
    at = AgentTasks.new(repo)
    _, _, diagnostics = at.process_workflows("/a\n/b")
    # should detect conflicting variable assignments
    assert_includes diagnostics, 'Conflicting assignment for VAR'
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_assignment_with_appends
    repo, remote = setup_repo(:git)
    create_workflow(repo, 'set', "#!/bin/sh\necho '@agents-setup VAR=base'")
    create_workflow(repo, 'add', "#!/bin/sh\necho '@agents-setup VAR+=extra'")
    at = AgentTasks.new(repo)
    _, env1, diagnostics1 = at.process_workflows("/set\n/add")
    # direct assignment followed by append should combine values
    assert_empty diagnostics1
    assert_equal 'base,extra', env1['VAR']

    _, env2, diagnostics2 = at.process_workflows("/add\n/set")
    # append before assignment should yield the same result
    assert_empty diagnostics2
    assert_equal 'base,extra', env2['VAR']

    create_workflow(repo, 'set_dup', "#!/bin/sh\necho '@agents-setup VAR=base'")
    _, env3, diagnostics3 = at.process_workflows("/set\n/set_dup\n/add")
    # duplicate direct assignment should not cause diagnostics
    assert_empty diagnostics3
    assert_equal 'base,extra', env3['VAR']
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_append_only_combines_values
    repo, remote = setup_repo(:git)
    create_workflow(repo, 'add1', "#!/bin/sh\necho '@agents-setup VAR+=one'")
    create_workflow(repo, 'add2', "#!/bin/sh\necho '@agents-setup VAR+=two'")
    at = AgentTasks.new(repo)
    _, env1, diagnostics1 = at.process_workflows("/add1\n/add2")
    # multiple append directives should accumulate
    assert_empty diagnostics1
    assert_equal 'one,two', env1['VAR']

    _, env2, diagnostics2 = at.process_workflows("/add1\n/add1")
    # duplicate append values should be deduplicated
    assert_empty diagnostics2
    assert_equal 'one', env2['VAR']
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end

  def test_workflow_command_failure_reports_stderr
    repo, remote = setup_repo(:git)
    create_workflow(repo, 'fail', "#!/bin/sh\necho boom >&2\nexit 1")
    at = AgentTasks.new(repo)
    _, _, diagnostics = at.process_workflows("Some text\n/fail   ")
    # stderr from failed command should appear in diagnostics
    assert(diagnostics.any? { |l| l.start_with?('$ fail') && l.include?('boom') })
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end
end
