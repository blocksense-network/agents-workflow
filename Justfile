# Run the test suite

test:
    ruby -Itest -e 'Dir["test/test_*.rb"].each { |f| require File.expand_path(f) }'

# Lint the Ruby codebase
lint:
    rubocop

# Build and publish the gem
publish-gem:
    gem build agent-task.gemspec && gem push agent-task-*.gem
