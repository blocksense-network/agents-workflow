# frozen_string_literal: true

# Pry-based debugging helper for Ruby debugging in VS Code
require 'pry'

module PryDebug
  def self.setup
    # Configure Pry for better debugging experience
    Pry.config.pager = false # Disable pager for VS Code terminal
    Pry.config.editor = proc { |file, line| "code --goto #{file}:#{line}" }

    # Custom prompt
    Pry.config.prompt = Pry::Prompt.new(
      'debug',
      'Debug prompt',
      [
        proc { |_context, _nesting, _pry_instance| '🔍 debug> ' },
        proc { |_context, _nesting, _pry_instance| '🔍 debug* ' }
      ]
    )
  end
end

# Initialize Pry configuration
PryDebug.setup

# Convenience methods
def debug_here(message = nil)
  location = defined?(self.class) ? self.class.to_s : 'main'
  method_name = caller_locations(1, 1).first.label
  full_message = message || "#{location}##{method_name}"
  PryDebug.break_here(full_message)
end

def pry_break(message = 'Manual breakpoint')
  PryDebug.break_here(message)
end

# Add debugging method to all objects
class Object
  def debug_inspect(label = 'Object')
    puts "🔍 #{label}: #{inspect}"
    puts "   Class: #{self.class}"
    puts "   Methods: #{methods(false).sort}" if respond_to?(:methods)
    self
  end
end
