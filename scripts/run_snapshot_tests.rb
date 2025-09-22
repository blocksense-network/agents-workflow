#!/usr/bin/env ruby
# frozen_string_literal: true

require 'minitest/autorun'

# Custom reporter for better output
class VerboseProgressReporter < Minitest::ProgressReporter
  def record(result)
    super
    return unless result.failure && !result.skipped?

    puts "\n#{'=' * 80}"
    puts "FAILURE: #{result.class}##{result.name}"
    puts '=' * 80
    puts result.failure.message
    puts result.failure.backtrace.join("\n") if result.failure.respond_to?(:backtrace) && result.failure.backtrace
    puts '=' * 80
    puts
  end
end

Minitest.reporter = VerboseProgressReporter.new

# Load only snapshot test files
snapshot_test_files = Dir['legacy/ruby/test/snapshot/test_*.rb'].sort
snapshot_test_files.each do |test_file|
  puts "Loading: #{File.basename(test_file)}"
  require File.expand_path(test_file)
end
