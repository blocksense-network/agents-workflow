# Test guidelines

- Always include a brief comment before each assertion explaining why the assertion is made. This helps future contributors understand the intent of the test.

All tests should use filesystem-based Git remotes instead of internet URLs to ensure tests can run offline without network dependencies. When testing remote functionality, create temporary local bare repositories.
