# Test guidelines

- Always include a brief comment before each assertion explaining why the assertion is made. This helps future contributors understand the intent of the test.

Tests using `start_agent_task` should use filesystem-based Git remotes instead of internet URLs to ensure they can run offline without network dependencies. To simulate a remote repos, create temporary local bare repositories.
