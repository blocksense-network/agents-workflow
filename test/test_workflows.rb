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
    script = File.join(wf_dir, 'hello')
    File.write(script, "#!/bin/sh\necho hello\necho '@agents-setup FOO=bar'")
    FileUtils.chmod(0o755, script)

    status, = run_agent_task(repo, branch: 'feat', prompt: "/hello\n@agents-setup BAZ=1", push_to_remote: false)
    assert_equal 0, status.exitstatus, 'agent-task should succeed'

    VCSRepo.new(repo).checkout_branch('feat')
    _, output = run_get_task(repo)
    assert_includes output, 'hello', 'workflow output missing'
    refute_includes output, '@agents-setup', 'setup directive should not appear'

    cmd = [RepoTestHelper::GET_TASK, '--get-setup-env']
    env_output = IO.popen(cmd, chdir: repo, &:read)
    status2 = $CHILD_STATUS
    assert_equal 0, status2.exitstatus, 'get-task --get-setup-env failed'
    lines = env_output.split("\n")
    assert_includes lines, 'FOO=bar', 'env from workflow missing'
    assert_includes lines, 'BAZ=1', 'env from task missing'
  ensure
    FileUtils.remove_entry(repo) if repo && File.exist?(repo)
    FileUtils.remove_entry(remote) if remote && File.exist?(remote)
  end
end
