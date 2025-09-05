### Mermaid Diagram Guidelines (authoring)

- Use descriptive node IDs (e.g., `are_we_inside_repo`) instead of single letters.
- Quote labels with spaces/parentheses: `["Label (with details)"]`.
- Quote decision nodes that include punctuation: `{ "Question?" }`.
- Terminal nodes must have no outgoing edges; name them clearly (e.g., `done`, `exit_error`).
- Close code fences correctly: start with ```mermaid and end with ``` on its own line.

# Agents Workflow Specs

This folder contains a work-in-progress specification for the next iteration of the agents-workflow product.

The main ideas of the product are described in the [Product One Pager](../docs/Product%20One%20Pager.md) document. Please read it.

Currently, the spec is not finalized and we are not ready to start the implementation of the software. The folder `specs/Initial Developer Input` contains files that should be treated as a ground truth for the specification effort. You are expected to work on the markdown files in the `specs/Public` folder, which should detail everything with a much higher precision, but please note that not all information in them has been fully reviewed by the development team yet.

Files in the public folder should never refer to documents in the `specs/Initial Developer Input` folder or the `specs/Research` folder, where we put preliminary-research findings that also haven't been fully vetted.

Your goal is to build a very comprehensive specification, meeting the goals stated in the initial developer input and expanding upon them with solid research and engineering. You may use the information provided in the preliminary research findings, but please verify it, potentially by building a small PoC programs.

The public spec should be a stand-alone document that never references other folders. In other words, your job is to transform the content from the other folders into a high-quality spec.

For each file in the `spec/Public` folder, there will be a corresponding file in the `spec/Implementation Progress` folder. This is a place to store information regarding what was already prototyped or implemented for production use. The files in this folder should contain references to source code files that are a good starting point for someone who wants to see the code behind the spec.

Some of the markdown files have standardized Obsidian headers indicating their current review status. Avoid modifying files with status "Reviewed" or "Final" unless explicitly asked.

## Specs Maintenance

- Before committing any change to the `specs/` folder, run `just lint-specs` from the project root. This performs Markdown linting, link checking, spell checking, prose/style linting, and Mermaid diagram validation.

If the pre-commit hook blocks your commit, run `just lint-specs`, address the reported issues, and commit again.

## CLI Documentation Guidelines

### Command Parameter Formatting

When documenting CLI commands with multiple parameters, use the following format to improve readability in Git PRs and general maintainability:

**Good Format:**

```
aw command [OPTIONS] [ARGUMENTS]

OPTIONS:
  --option1 <value>           Description of option1
  --option2                   Description of option2
  --long-option-name <type>   Description of long option

ARGUMENTS:
  ARGUMENT1                   Description of required argument
  [ARGUMENT2]                 Description of optional argument
```

**Bad Format (hard to review in PRs):**

```
- `aw command [--option1 <value>] [--option2] [--long-option-name <type>] [argument1] [argument2]`
```

**Benefits of the good format:**

- Each parameter appears on its own line, making diffs cleaner in PRs
- Easier to spot additions, removals, or changes to individual parameters
- Better alignment and readability
- Consistent with standard help screen formatting
- Easier to maintain and update parameter descriptions

**When to use this format:**

- Commands with 3+ parameters
- Complex commands where parameter descriptions are important
- Any command where the single-line format becomes unwieldy

**When single-line format is acceptable:**

- Simple commands with 1-2 parameters
- Commands where brevity is more important than detailed formatting
