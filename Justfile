# Run the test suite

test:
    ruby -Itest test/run_tests_shell.rb

# Lint the Ruby codebase
lint:
    rubocop

# Build and publish the gem
publish-gem:
    gem build agent-task.gemspec && gem push agent-task-*.gem
