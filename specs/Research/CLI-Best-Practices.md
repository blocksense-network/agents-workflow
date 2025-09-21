# Standards for CLI Help Screens and Rust CLI Libraries

## Common Conventions for CLI Help Text

There is **no single official standard** for the format of command-line utility help screens. However, most CLI tools follow widely accepted **conventions and patterns**[\[1\]](https://bettercli.org/design/cli-help-page/#:~:text=Formatting%20and%20displaying%20CLI%20Help,). Typically, a help message includes:

- **Brief Description** – A one-line summary of what the program does.

- **Usage Synopsis** – A line (or lines) showing how to invoke the command. This usually follows a convention where _required arguments_ are shown as \<ARG\>, _optional arguments_ are in square brackets (e.g. \[ARG\]), and mutually exclusive choices are separated by pipes (e.g. {start|stop})[\[2\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=Typically%2C%20your%20help%20output%20should,include). For example, a usage line might look like:

- Usage: mytool \[OPTIONS\] \<input-file\> \[output-file\]

- Here \<input-file\> is required, output-file is optional, and \[OPTIONS\] indicates where flags can go.

- **Options/Flags List** – A nicely formatted list of available flags and options. Common practice is to show the short flag (like \-v) together with the long flag (\--verbose) in one line, followed by a description[\[3\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=case%20%2A%20A%20nicely,config%20files%20or%20environment%20variables). If an option takes a value, the value’s placeholder is shown (e.g. \--output \<FILE\>). Good help screens also indicate default values or possible value choices for options when applicable[\[3\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=case%20%2A%20A%20nicely,config%20files%20or%20environment%20variables).

- **Additional Info** – Some tools mention environment variables or config files that affect behavior, and many indicate where to find more documentation (e.g. a manual page or website)[\[3\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=case%20%2A%20A%20nicely,config%20files%20or%20environment%20variables). It’s also a common convention that both \-h and \--help trigger the help screen, and that running the tool with incorrect arguments will display the usage message as a hint to the user[\[4\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=,detailed%20help%20can%20be%20found).

These conventions are reflected in guidelines like the **man page** format and various CLI style guides. For instance, the **Docopt** specification attempted to formalize a standard syntax for usage docs and argument parsing. Docopt prescribes a usage section with the notations described above (optional items in \[\], alternatives in ()|, etc.), and even allows the help text to serve as the argument parser definition[\[5\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=157). In summary, while there isn’t an official standard, following the established patterns (description, usage, options, examples, etc.) will meet user expectations for a “standard” help screen.

## Rust Libraries for Polished CLI Help (with Colors & Completions)

If you’re looking for a Rust library that handles command-line parsing **and** produces polished, user-friendly help screens, the [**Clap**](https://crates.io/crates/clap) crate is a top choice. Clap is the most popular Rust CLI parser, and it puts a lot of effort into generating high-quality help text out-of-the-box[\[6\]](https://docs.rs/clap/latest/clap/#:~:text=,breaking%20changes%20in%20large%20releases). With Clap, you get:

- **Automatic Help Generation:** Define your command-line options and arguments in code (using either a builder API or derive macros), and Clap will automatically generate the \--help text for you. This includes a usage line and an option/flag list formatted as described above, with any default values or possible options included. You can customize the help message content (sections like description, examples, etc.) via attributes or builder methods, but Clap’s defaults already follow good conventions.

- **Styled Output (Colors/Bold):** Clap’s help output is designed to be easy to read. Earlier versions of Clap colorized the help screen by default, using colors to distinguish flags, argument names, and so on. In Clap 3.x, for example, option names might appear in **green** and placeholders in _yellow_, etc. In Clap 4.x, the default styling was toned down to use bold/underline for emphasis (for broader compatibility), but you can easily enable coloring or define a custom style theme if desired[\[7\]](https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help#:~:text=24)[\[8\]](https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help#:~:text=Color%20defaults%20were%20,less%20pleasant%20for%20most%20people). Clap’s API provides a Styles builder where you can set the text style or color for different parts of the help message (usage, headers, flags, errors, hints, etc.), allowing you to achieve a very polished, visually distinctive look. In short, Clap supports colored output and other text attributes – so you can highlight option names, parameters, and descriptions in different colors or styles for clarity[\[6\]](https://docs.rs/clap/latest/clap/#:~:text=,breaking%20changes%20in%20large%20releases).

- **User-Friendly Features:** A polished CLI experience is more than just looks. Clap also provides quality-of-life features like **automatic error messages and suggestions**. For example, if a user typos a subcommand or flag, Clap will catch it and display an error _with a “did you mean ...?” suggestion_. It also handles missing required arguments by showing the usage and indicating which ones are missing. These little touches (which Clap enables by default) make the CLI feel professional and user-friendly[\[6\]](https://docs.rs/clap/latest/clap/#:~:text=,breaking%20changes%20in%20large%20releases).

**Shell Auto-Completion Support:** Clap has excellent support for shell completions. Using the companion crate [**clap_complete**](https://crates.io/crates/clap_complete) (maintained by the Clap team), you can automatically generate completion scripts for all mainstream shells like **Bash, Zsh, and Fish** (as well as others like PowerShell and Elvish)[\[9\]](https://docs.rs/clap_complete/latest/clap_complete/aot/enum.Shell.html#:~:text=,Zsh). This means you can provide tab-completion for your tool’s subcommands, flags, and even option values. The typical approach is either:

- Generate the completion script at build-time or runtime and instruct the user to install/source it in their shell. For example, you might include a hidden command or flag (e.g. mytool \--generate-completion bash) that prints the completion script to stdout, which the user can redirect to a file in their bash_completion.d. Clap will take care of listing all your subcommands and options in that script.

- **Value Hints and Possible Values:** Clap can also embed hints for completing common value types. For instance, if an argument is a file path, you can tag it with ValueHint::AnyPath, and the generated completion script will tell the shell to autocomplete using file names[\[10\]](https://docs.rs/clap_complete/latest/clap_complete/#:~:text=.arg%28Arg%3A%3Anew%28,.action%28ArgAction%3A%3ASet%29%20.value_parser%28value_parser%21%28Shell%29%29%29). You can also provide a fixed set of possible values for an option (e.g. a list of modes or formats), and Clap will include those in the completion suggestions.

Using Clap’s completion support, it’s straightforward to target Bash, Zsh, and Fish with minimal effort – you get completion scripts for each of them generated from the same CLI definition.

**Custom/Dynamic Completions:** If you have arguments whose valid values are dynamic (for example, names of resources that the program can query at runtime), you might need to incorporate custom logic into the completion system. There are a couple of ways to handle this:

- _Leverage Clap’s evolving completer API:_ The Clap developers are working on a more dynamic completion API. The clap_complete crate includes an (unstable) **engine** for “native” shell completion, where your program itself can be invoked by the shell to provide completions[\[11\]](https://docs.rs/clap_complete/latest/clap_complete/#:~:text=aot%20%20Prebuilt%20completions%20,Deprecated%2C%20see%20%2063%20shells). In the future, this will allow you to write a custom completion handler (e.g. a function that lists resource names by querying a server or database) and have the shell call into your binary for suggestions. This feature is under active development, but it shows that Clap’s ecosystem is moving toward first-class dynamic auto-completion support.

- _Use a custom completion script or helper:_ Another approach is to write the completion logic yourself using shell facilities or a helper crate. For example, Bash’s complete built-in lets you designate a command to produce completions. You could have your program detect a special environment variable or subcommand (like \_\_complete) and output suggestions. The **shell_completion** crate is a community library that provides low-level primitives to help implement such logic in Rust[\[12\]](https://www.joshmcguigan.com/blog/shell-completions-pure-rust/#:~:text=Introducing%20). Using shell_completion, you can write a Rust function to generate completion candidates (possibly calling into your program’s internal APIs to list resources), and tie it into the shell’s completion system. This gives you full control to produce context-aware suggestions (similar to how git or kubectl do completions by calling themselves).

In summary, **Clap** (with its derive macros and related tools) is a highly recommended solution for building a CLI with professional-quality help text. It follows common help-screen conventions automatically and allows extensive customization of the output format (including colors and layout). Clap will also handle generating shell completion scripts for Bash, Zsh, Fish and others out-of-the-box[\[6\]](https://docs.rs/clap/latest/clap/#:~:text=,breaking%20changes%20in%20large%20releases)[\[9\]](https://docs.rs/clap_complete/latest/clap_complete/aot/enum.Shell.html#:~:text=,Zsh). For advanced auto-completion needs (dynamic completions), you can integrate custom logic either through Clap’s facilities or external helpers. This way, you can deliver a polished command-line interface with colorful, easy-to-read help screens and convenient tab-completion support on all major shells.

**Sources:**

- CLI help screen conventions and guidelines[\[1\]](https://bettercli.org/design/cli-help-page/#:~:text=Formatting%20and%20displaying%20CLI%20Help,)[\[2\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=Typically%2C%20your%20help%20output%20should,include)[\[3\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=case%20%2A%20A%20nicely,config%20files%20or%20environment%20variables)

- Clap crate documentation (features like colored help and shell completions)[\[6\]](https://docs.rs/clap/latest/clap/#:~:text=,breaking%20changes%20in%20large%20releases)

- Clap Complete documentation (supported shells for auto-completion)[\[9\]](https://docs.rs/clap_complete/latest/clap_complete/aot/enum.Shell.html#:~:text=,Zsh)

- Discussion of advanced shell completion techniques in Rust[\[12\]](https://www.joshmcguigan.com/blog/shell-completions-pure-rust/#:~:text=Introducing%20)

---

[\[1\]](https://bettercli.org/design/cli-help-page/#:~:text=Formatting%20and%20displaying%20CLI%20Help,) CLI Help pages \- Better CLI

[https://bettercli.org/design/cli-help-page/](https://bettercli.org/design/cli-help-page/)

[\[2\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=Typically%2C%20your%20help%20output%20should,include) [\[3\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=case%20%2A%20A%20nicely,config%20files%20or%20environment%20variables) [\[4\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=,detailed%20help%20can%20be%20found) [\[5\]](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text#:~:text=157) Is there a "standard" format for command line/shell help text? \- Stack Overflow

[https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text](https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text)

[\[6\]](https://docs.rs/clap/latest/clap/#:~:text=,breaking%20changes%20in%20large%20releases) clap \- Rust

[https://docs.rs/clap/latest/clap/](https://docs.rs/clap/latest/clap/)

[\[7\]](https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help#:~:text=24) [\[8\]](https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help#:~:text=Color%20defaults%20were%20,less%20pleasant%20for%20most%20people) rust \- clap.rs not printing colors during \`--help\` \- Stack Overflow

[https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help](https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help)

[\[9\]](https://docs.rs/clap_complete/latest/clap_complete/aot/enum.Shell.html#:~:text=,Zsh) Shell in clap_complete::aot \- Rust

[https://docs.rs/clap_complete/latest/clap_complete/aot/enum.Shell.html](https://docs.rs/clap_complete/latest/clap_complete/aot/enum.Shell.html)

[\[10\]](https://docs.rs/clap_complete/latest/clap_complete/#:~:text=.arg%28Arg%3A%3Anew%28,.action%28ArgAction%3A%3ASet%29%20.value_parser%28value_parser%21%28Shell%29%29%29) [\[11\]](https://docs.rs/clap_complete/latest/clap_complete/#:~:text=aot%20%20Prebuilt%20completions%20,Deprecated%2C%20see%20%2063%20shells) clap_complete \- Rust

[https://docs.rs/clap_complete/latest/clap_complete/](https://docs.rs/clap_complete/latest/clap_complete/)

[\[12\]](https://www.joshmcguigan.com/blog/shell-completions-pure-rust/#:~:text=Introducing%20) Shell Completions in Pure Rust

[https://www.joshmcguigan.com/blog/shell-completions-pure-rust/](https://www.joshmcguigan.com/blog/shell-completions-pure-rust/)
