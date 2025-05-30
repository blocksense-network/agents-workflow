# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name          = 'agent-task'
  spec.version       = '0.1.0'
  spec.authors       = ['Blocksense']
  spec.summary       = 'Utility to start tasks for coding agents.'
  spec.files         = Dir['bin/*', 'lib/**/*.rb', 'LICENSE', 'README.md', 'codex-setup']
  spec.executables   = Dir['bin/*'].select { |f| File.file?(f) }.map { |f| File.basename(f) }
  spec.require_paths = ['lib']
  spec.required_ruby_version = '>= 3.0.0'
  spec.metadata['rubygems_mfa_required'] = 'true'
end
