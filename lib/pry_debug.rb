# Pry-based debugging helper for Ruby debugging in VS Code
require 'pry'

module PryDebug
  def self.setup
    # Configure Pry for better debugging experience
    Pry.config.pager = false # Disable pager for VS Code terminal
    Pry.config.editor = proc { |file, line| "code --goto #{file}:#{line}" }

    # Custom prompt
    Pry.config.prompt = Pry::Prompt.new(
      "debug",
      "Debug prompt",
      [
        proc { |context, nesting, pry_instance| "🔍 debug> " },
        proc { |context, nesting, pry_instance| "🔍 debug* " }
      ]
    )
  end

  def self.break_here(message = "Debug breakpoint")
    puts "\n" + "="*60
    puts "🔍 DEBUG: #{message}"
    puts "📍 Location: #{caller(1,1).first}"
    puts "="*60
    puts "Available commands:"
    puts "  exit       - Continue execution"
    puts "  ls         - List methods and variables"
    puts "  whereami   - Show current code context"
    puts "  help       - Show all Pry commands"
    puts "="*60

    binding.pry

    puts "Continuing execution..."
  end
end

# Initialize Pry configuration
PryDebug.setup

# Convenience methods
def debug_here(message = nil)
  location = defined?(self.class) ? "#{self.class}" : "main"
  method_name = caller_locations(1,1).first.label
  full_message = message || "#{location}##{method_name}"
  PryDebug.break_here(full_message)
end

def pry_break(message = "Manual breakpoint")
  PryDebug.break_here(message)
end

# Add debugging method to all objects
class Object
  def debug_inspect(label = "Object")
    puts "🔍 #{label}: #{self.inspect}"
    puts "   Class: #{self.class}"
    puts "   Methods: #{self.methods(false).sort}" if respond_to?(:methods)
    self
  end
end
