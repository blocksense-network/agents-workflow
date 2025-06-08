#!/usr/bin/env ruby
# frozen_string_literal: true

require 'agent_task'

if ARGV.first == 'setup'
  ARGV.shift
  AgentTask::CLI.run_setup(ARGV)
else
  AgentTask::CLI.start_task(ARGV)
end
