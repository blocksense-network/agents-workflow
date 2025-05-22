You can execute the test suite with `just test`.

Please note that `start-task` command will launch an editor.

When adding tests that use this command, I suggest overriding the `EDITOR` env variable with a simple program that writes the required task definition to the supplied path and them immediately exits.
