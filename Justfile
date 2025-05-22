test:
        ruby -Itest -e 'Dir["test/test_*.rb"].each { |f| require File.expand_path(f) }'
