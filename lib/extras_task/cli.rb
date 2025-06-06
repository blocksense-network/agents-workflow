# frozen_string_literal: true

require_relative '../extras_installer'

module ExtrasTask
  # CLI exposes the extras installation functionality as callable methods
  # following the same patterns as AgentTask::CLI
  module CLI
    module_function

    # Main entry point for the install-extras CLI command
    # Parses command line arguments and environment variables
    def install_extras(args = [], stdout: $stdout)
      require 'optparse'

      options = {}
      OptionParser.new do |opts|
        opts.banner = 'Usage: install-extras [options]'
        opts.on('--test-mode', 'Run in test mode (mock installations)') do
          options[:test_mode] = true
        end
        opts.on('--clean', 'Clean installation markers') do
          options[:clean] = true
        end
        opts.on('--help', 'Show help information') do
          options[:help] = true
        end
        opts.on('--extras=COMPONENTS', 'Specify components to install') do |val|
          options[:extras] = val
        end
      end.parse!(args)

      # Handle special commands
      if options[:help]
        ExtrasInstaller.help
        return
      end

      if options[:clean]
        ExtrasInstaller.clean_markers
        return
      end

      # Determine extras source (command line option takes precedence)
      extras_string = options[:extras] || ENV.fetch('EXTRAS', '')

      # Handle legacy NIX=1 environment variable
      extras_string = 'nix' if extras_string.strip.empty? && ENV['NIX'] == '1'

      if extras_string.strip.empty?
        stdout.puts 'No extras specified in EXTRAS environment variable'
        stdout.puts "Available extras: #{ExtrasInstaller::VALID_COMPONENTS.join(', ')}"
        stdout.puts "Example: EXTRAS='nix,direnv' #{$PROGRAM_NAME}"
        return
      end

      begin
        # Check for test mode from command line flag or environment variable
        test_mode = options[:test_mode] || ENV['TEST_MODE'] == '1'
        installer = ExtrasInstaller.new(extras_string, test_mode: test_mode)
        installer.install_all_direct
      rescue ExtrasError => e
        stdout.puts "ERROR: #{e.message}"
        exit 1
      rescue StandardError => e
        stdout.puts "ERROR: Unexpected error: #{e.message}"
        stdout.puts e.backtrace.join("\n") if ENV['DEBUG']
        exit 1
      end
    end

    # Legacy compatibility method for NIX=1 environment variable
    # This maintains backward compatibility with existing scripts
    def handle_legacy_nix
      return unless ENV['NIX'] == '1'

      puts 'Legacy NIX=1 detected, using EXTRAS framework'
      ENV['EXTRAS'] = 'nix'
      install_extras
    end
  end
end
