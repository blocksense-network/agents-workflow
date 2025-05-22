Gem::Specification.new do |spec|
  spec.name          = 'agent-task'
  spec.version       = '0.1.0'
  spec.authors       = ['Agents Workflow Maintainers']
  spec.summary       = 'Utility to start tasks for coding agents.'
  spec.files         = Dir['bin/*', 'bin/lib/**/*.rb', 'lib/**/*.rb', 'LICENSE', 'README.md']
  spec.executables   = ['agent-task']
  spec.require_paths = ['bin/lib', 'lib']
end
