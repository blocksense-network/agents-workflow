# frozen_string_literal: true

require 'fileutils'

# Custom exception classes for extras operations
class ExtrasError < StandardError; end
class InvalidComponentError < ExtrasError; end
class DependencyError < ExtrasError; end
class InstallationError < ExtrasError; end

# The ExtrasInstaller provides a Ruby interface for installing individual components
# Dependency management is handled by Rake's built-in dependency system
class ExtrasInstaller
  attr_reader :test_mode, :components

  # Valid components that can be installed
  VALID_COMPONENTS = %w[nix direnv cachix].freeze

  # Components that require nix to be available
  NIX_DEPENDENT_COMPONENTS = %w[direnv cachix].freeze

  # Path configuration relative to the project root
  AGENTS_WORKFLOW_DIR = File.expand_path('..', __dir__)
  INSTALL_DIR = File.join(AGENTS_WORKFLOW_DIR, 'install')
  MARKER_DIR = File.join(AGENTS_WORKFLOW_DIR, '.install-markers')

  def initialize(extras_string = nil, **options)
    @test_mode = options.fetch(:test_mode, false)
    @components = parse_extras(extras_string || ENV.fetch('EXTRAS', ''))
    validate_components!

    # Ensure marker directory exists
    FileUtils.mkdir_p(MARKER_DIR)
  end

  # Parse EXTRAS string with flexible separators
  # Supports: comma, semicolon, plus, pipe, and space
  def parse_extras(extras_string)
    return [] if extras_string.nil? || extras_string.strip.empty?

    # Replace various separators with spaces, then clean up
    extras_string
      .gsub(/[,;+|]/, ' ')       # Replace separators with spaces
      .squeeze(' ')              # Collapse multiple spaces
      .strip                     # Remove leading/trailing whitespace
      .split # Split on remaining spaces
      .reject(&:empty?)          # Remove empty elements
      .uniq                      # Remove duplicates
      .sort                      # Sort for consistent ordering
  end

  # Validate that all components are supported
  def validate_components!
    invalid_components = @components - VALID_COMPONENTS

    return if invalid_components.empty?

    raise InvalidComponentError,
          "Invalid components specified: #{invalid_components.join(', ')}. " \
          "Valid components are: #{VALID_COMPONENTS.join(', ')}"
  end

  # Check if a component is already installed using marker files
  def component_installed?(component)
    marker_file = File.join(MARKER_DIR, "#{component}.done")
    File.exist?(marker_file)
  end

  # Install a single component (should be called with dependencies already handled by Rake)
  def install_component(component, nix_will_be_available: false)
    return if component_installed?(component)

    install_script = File.join(INSTALL_DIR, "install-#{component}")

    raise InstallationError, "Install script not found: #{install_script}" unless File.exist?(install_script)

    # Prepare environment for the install script
    env = ENV.to_h.dup
    env['TEST_MODE'] = '1' if @test_mode

    # Set MOCK_NIX_AVAILABLE if nix is being installed, already installed, or if this component depends on nix
    if @test_mode && (component == 'nix' || nix_will_be_available || NIX_DEPENDENT_COMPONENTS.include?(component))
      env['MOCK_NIX_AVAILABLE'] = '1'
    end

    puts "Installing #{component}..."

    # Execute the install script
    success = system(env, 'bash', install_script)

    raise InstallationError, "Failed to install component: #{component}" unless success

    # Create marker file to indicate successful installation
    marker_file = File.join(MARKER_DIR, "#{component}.done")
    File.write(marker_file, Time.now.to_s)
  end

  # Parse and return the requested components for Rake to handle
  # This method is used by Rakefile.extras to get the list of components to install
  def install_all
    return [] if @components.empty?

    puts "Processing EXTRAS: #{@components.join(' ')}"
    @components
  end

  # Simple installation for when called directly (bypassing Rake)
  def install_all_direct
    return if @components.empty?

    puts "Processing EXTRAS: #{@components.join(' ')}"
    puts 'Warning: Installing directly without Rake dependency management'

    # Set up global environment for the installation session
    nix_will_be_available = @components.include?('nix') || component_installed?('nix')
    ENV['MOCK_NIX_AVAILABLE'] = '1' if @test_mode && nix_will_be_available

    begin
      # Install in a reasonable order (nix first, then others)
      ordered_components = @components.sort_by { |comp| comp == 'nix' ? 0 : 1 }

      ordered_components.each do |component|
        if component_installed?(component)
          puts "#{component} is already installed (marker found)"
        else
          install_component(component, nix_will_be_available: nix_will_be_available)
        end
      end

      puts ''
      puts "Successfully installed all requested extras: #{ordered_components.join(' ')}"
    ensure
      # Clean up global environment
      ENV.delete('MOCK_NIX_AVAILABLE') if @test_mode
    end
  end

  # Clean all installation markers (for testing/debugging)
  def self.clean_markers
    marker_dir = File.join(AGENTS_WORKFLOW_DIR, '.install-markers')
    return unless Dir.exist?(marker_dir)

    FileUtils.rm_rf(marker_dir)
    puts 'Cleaned installation markers'
    puts 'Note: This only removes markers, not the actual installed software'
  end

  # Display help information
  def self.help
    puts 'ExtrasInstaller - Component installation framework'
    puts ''
    puts 'Available components:'
    puts '  nix     - Nix package manager (foundational)'
    puts '  direnv  - Directory-based environment management (requires nix)'
    puts '  cachix  - Binary cache service for Nix (requires nix)'
    puts ''
    puts 'Usage:'
    puts '  Set EXTRAS environment variable with component names'
    puts '  Supports various separators: comma, semicolon, plus, pipe, space'
    puts ''
    puts 'Examples:'
    puts "  EXTRAS='nix,direnv' ruby -Ilib bin/install-extras"
    puts "  EXTRAS='nix;direnv+cachix' ruby -Ilib bin/install-extras"
    puts ''
    puts 'Commands:'
    puts '  ExtrasInstaller.clean_markers  # Clean installation markers'
    puts '  ExtrasInstaller.help          # Show this help'
  end
end
