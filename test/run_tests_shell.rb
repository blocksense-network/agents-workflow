#!/usr/bin/env ruby
# frozen_string_literal: true

# Test runner script with buffered output - shell redirection approach
# Uses shell-level redirection to capture ALL subprocess output

require 'English'
require 'fileutils'
require 'time'
require 'shellwords'

# Create logs directory
logs_dir = File.join(__dir__, 'logs')
FileUtils.mkdir_p(logs_dir)

# Generate log file path
timestamp = Time.now.strftime('%Y%m%d_%H%M%S')
log_file = File.join(logs_dir, "test_run_#{timestamp}.log")
temp_output = File.join(logs_dir, "temp_output_#{timestamp}.log")

class ShellTestRunner
  def initialize(log_file, temp_output)
    @log_file = log_file
    @temp_output = temp_output
    @start_time = Time.now

    # Initialize log file
    File.open(@log_file, 'w') do |f|
      f.puts "Test run started at #{@start_time}"
      f.puts '=' * 80
      f.puts
    end
  end

  def run
    puts "🚀 Running tests... (full output logged to #{@log_file})"
    puts

    # Create a simple test script that runs all tests
    test_script = create_test_script

    begin
      # Run the test script with shell redirection to capture ALL output
      # This includes all subprocess output from git, system calls, etc.
      escaped_script = Shellwords.escape(test_script)
      escaped_output = Shellwords.escape(@temp_output)

      # Use shell redirection to capture stdout and stderr
      command = "ruby #{escaped_script} > #{escaped_output} 2>&1"

      success = system(command)
      exit_code = $CHILD_STATUS.exitstatus

      # Read the captured output
      output = File.exist?(@temp_output) ? File.read(@temp_output) : ''

      # Parse and display results
      parse_and_display_results(output, exit_code, success)

      # Return the exit code to be used by the main script
      exit_code
    ensure
      # Clean up temp files
      FileUtils.rm_f(test_script)
      FileUtils.rm_f(@temp_output)
    end
  end

  private

  def create_test_script
    script_path = File.join(__dir__, "temp_test_runner_#{Time.now.to_i}.rb")

    script_content = <<~RUBY
      #!/usr/bin/env ruby
      # frozen_string_literal: true

      # Change to the test directory to ensure proper loading
      Dir.chdir("#{__dir__}")

      require 'minitest/autorun'

      # Use a custom reporter that shows progress and failures clearly
      class VerboseProgressReporter < Minitest::ProgressReporter
        def record(result)
          super
          if result.failure && !result.skipped?
            puts "\n" + "=" * 80
            puts "FAILURE: \#{result.class}#\#{result.name}"
            puts "=" * 80
            puts result.failure.message
            if result.failure.respond_to?(:backtrace) && result.failure.backtrace
              puts result.failure.backtrace.join("\n")
            end
            puts "=" * 80
            puts
          end
        end
      end

      Minitest.reporter = VerboseProgressReporter.new

      # Load all test files
      test_files = []
      test_files.concat(Dir["test_*.rb"].sort)
      test_files.concat(Dir["snapshot/test_*.rb"].sort)

      test_files.each do |test_file|
        puts "Loading: \#{File.basename(test_file)}"
        require File.expand_path(test_file)
      end
    RUBY

    File.write(script_path, script_content)
    File.chmod(0o755, script_path)
    script_path
  end

  def parse_and_display_results(output, exit_code, success)
    # Log the full output
    File.open(@log_file, 'a') do |f|
      f.puts "Exit code: #{exit_code}"
      f.puts "Success: #{success}"
      f.puts '-' * 60
      f.puts 'FULL OUTPUT:'
      f.puts output
      f.puts '-' * 60
    end

    # Extract and display the progress line
    lines = output.split("\n")
    progress_lines = lines.select { |line| line.match(/^[.FS]+$/) }

    if progress_lines.any?
      puts '🧪 Test Progress:'
      progress_lines.each { |line| puts "   #{line}" }
    else
      puts '❓ No test progress found in output'
    end

    puts

    # Extract failing test names from minitest output
    failing_tests = extract_failing_test_names(output)
    if failing_tests.any?
      puts '❌ FAILING TESTS:'
      puts "🔴 #{'=' * 58}"
      failing_tests.each_with_index do |test_info, index|
        emoji = test_info[:type] == 'Failure' ? '💥' : '🚨'
        puts "#{emoji} #{index + 1}. #{test_info[:type]}: #{test_info[:name]}"
        puts "   📝 #{test_info[:message]}" if test_info[:message] && !test_info[:message].empty?
      end
      puts "🔴 #{'=' * 58}"
      puts
    end

    # Extract test statistics
    stats = extract_test_stats(output)
    show_summary(stats, exit_code)
  end

  def extract_failing_test_names(output)
    failing_tests = []
    lines = output.split("\n")

    # Look for minitest failure/error format
    # Example: "1) Failure:\nFollowUpGitTest#test_nested_branch_tasks [test/test_follow_up_tasks.rb:58]:"
    # Example: "12) Error:\nStartTaskGitTest#test_dirty_repo_staged:"
    i = 0
    while i < lines.length
      line = lines[i]

      # Check for numbered failure/error
      if line.match(/^\s*\d+\)\s+(Failure|Error):\s*$/)
        type = ::Regexp.last_match(1)
        i += 1

        # Next line should contain the test name
        if i < lines.length
          if lines[i].match(/^(.+?)#(.+?)\s+\[(.+?):(\d+)\]:?/)
            # Format with file reference: "ClassName#method_name [file:line]:"
            class_name = ::Regexp.last_match(1)
            method_name = ::Regexp.last_match(2)
            file_name = ::Regexp.last_match(3)
            line_number = ::Regexp.last_match(4)

            # Look for the error message in subsequent lines
            message_lines = []
            j = i + 1
            while j < lines.length &&
                  !lines[j].match(/^\s*\d+\)\s+(Failure|Error):\s*$/) &&
                  !lines[j].match(/^\d+\s+runs?,/)
              # Skip empty lines and very long lines (likely stack traces)
              message_lines << lines[j].strip unless lines[j].strip.empty? || lines[j].length > 200
              j += 1
              break if message_lines.length >= 2 # Limit to first few lines
            end

            failing_tests << {
              type: type,
              name: "#{class_name}##{method_name}",
              file: file_name,
              line: line_number.to_i,
              message: message_lines.first || ''
            }
          elsif lines[i].match(/^(.+?)#(.+?):\s*$/)
            # Format without file reference: "ClassName#method_name:"
            class_name = ::Regexp.last_match(1)
            method_name = ::Regexp.last_match(2)

            # Look for the error message in subsequent lines
            message_lines = []
            j = i + 1
            while j < lines.length &&
                  !lines[j].match(/^\s*\d+\)\s+(Failure|Error):\s*$/) &&
                  !lines[j].match(/^\d+\s+runs?,/)
              # Skip empty lines and very long lines (likely stack traces)
              message_lines << lines[j].strip unless lines[j].strip.empty? || lines[j].length > 200
              j += 1
              break if message_lines.length >= 2 # Limit to first few lines
            end

            failing_tests << {
              type: type,
              name: "#{class_name}##{method_name}",
              file: 'unknown',
              line: 0,
              message: message_lines.first || ''
            }
          end
        end
      end
      i += 1
    end

    failing_tests
  end

  def extract_failure_sections(output)
    sections = []
    lines = output.split("\n")
    in_failure = false
    current_section = []

    lines.each do |line|
      if line.start_with?('=' * 80) && line.include?('FAILURE')
        in_failure = true
        current_section = [line]
      elsif in_failure && line.start_with?('=' * 80)
        current_section << line
        sections << current_section.join("\n")
        in_failure = false
        current_section = []
      elsif in_failure
        current_section << line
      end
    end

    # Add any remaining section
    sections << current_section.join("\n") if current_section.any?

    sections
  end

  def extract_test_stats(output)
    # Look for minitest summary line like: "59 runs, 198 assertions, 4 failures, 4 errors, 0 skips"
    stats_line = output.lines.find { |line| line.match(/\d+\s+runs?,.*assertions.*failures.*errors.*skips/) }

    if stats_line
      # Parse the summary line
      runs = stats_line.match(/(\d+)\s+runs?/)&.captures&.first.to_i
      failures = stats_line.match(/(\d+)\s+failures?/)&.captures&.first.to_i
      errors = stats_line.match(/(\d+)\s+errors?/)&.captures&.first.to_i
      skips = stats_line.match(/(\d+)\s+skips?/)&.captures&.first.to_i

      { runs: runs, failures: failures, errors: errors, skips: skips }
    else
      # Try to count from progress line if no summary found
      progress_lines = output.lines.select { |line| line.match(/^[.FS]+$/) }
      if progress_lines.any?
        progress = progress_lines.join
        runs = progress.length
        failures = progress.count('F')
        errors = 0 # Can't distinguish from progress alone
        skips = progress.count('S')

        { runs: runs, failures: failures, errors: errors, skips: skips }
      else
        { runs: 0, failures: 0, errors: 0, skips: 0 }
      end
    end
  end

  def show_summary(stats, exit_code)
    total_issues = stats[:failures] + stats[:errors]

    if total_issues.zero? && exit_code.zero?
      puts '✨ All tests passed! 🎉✨'
    else
      puts "⚠️  #{total_issues} test(s) failed."
    end

    puts
    puts '📊 Test Summary:'
    puts "  🏃 Total tests: #{stats[:runs]}"
    puts "  💥 Failures: #{stats[:failures]}"
    puts "  🚨 Errors: #{stats[:errors]}"
    puts "  ⏭️  Skips: #{stats[:skips]}"
    puts "  ⏱️  Duration: #{(Time.now - @start_time).round(2)}s"
    puts "  🚪 Exit code: #{exit_code}"
    puts
    puts "📄 Full test output available at: #{@log_file}"

    # Add summary to log file
    File.open(@log_file, 'a') do |f|
      f.puts
      f.puts '=' * 80
      f.puts "Test run completed at #{Time.now}"
      f.puts "Duration: #{(Time.now - @start_time).round(2)}s"
      f.puts "Total tests: #{stats[:runs]}"
      f.puts "Failures: #{stats[:failures]}"
      f.puts "Errors: #{stats[:errors]}"
      f.puts "Skips: #{stats[:skips]}"
      f.puts "Exit code: #{exit_code}"
      f.puts '=' * 80
    end
  end
end

# Run the shell-based test runner
runner = ShellTestRunner.new(log_file, temp_output)
exit_code = runner.run
exit(exit_code)
