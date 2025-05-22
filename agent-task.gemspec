# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name          = 'agent-task'
  spec.version       = '0.1.0'
  spec.authors       = ['Blocksense']
  spec.summary       = 'Utility to start tasks for coding agents.'
  spec.files         = Dir['bin/*', 'bin/lib/**/*.rb', 'lib/**/*.rb', 'LICENSE', 'README.md']
  spec.executables   = ['agent-task']
  spec.require_paths = ['bin/lib', 'lib']
  spec.required_ruby_version = '>= 3.0.0'
  spec.metadata['rubygems_mfa_required'] = 'true'
end
