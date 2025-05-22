# Testing your changes

- You can execute the test suite with `just test`.
- You can lint the codebase with `just lint`.

# Code quality guidelines

- Strive to achieve high code quality.
- Write secure code.
- Make sure the code is well tested and edge cases are covered. Design the code for testability and be extremely thorough.
- Write defensive code and make sure all potential errors are handled.
- Strive to write highly reusable code with routines that have high fan in and low fan out.
- Keep the code DRY.
- Aim for low coupling and high cohesion. Encapsulate and hide implementation details.

# Code commenting guidelines

- Document public APIs and complex modules.
- Maintain the comments together with the code to keep them meaningful and current.
- Comment intention and rationale, not obvious facts. Write self-documenting code.
- When implementing specific formats, standards or other specifications, make sure to
  link to relevant URLs that provide the necessary technical details.

# Writing git commit messages

- Use the convential commits style for the first line of the commit message.
- Write the summary that you are using in PRs in the git commit body.
