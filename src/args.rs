use std::sync::Arc;

use clap::Parser;

/// Command-line arguments for Hexmake
#[derive(Parser)]
#[command(version)]
#[command(arg_required_else_help = true)]
#[command(about = "Run a multi-step build with caching")]
#[command(
    long_about = r#"Hexmake runs a multi-step build using caching. You give it a file describing all
the possible build steps along with their inputs and outputs. The tool will then
chain them together to produce an output, use cached results from prior builds
when possible.
"#
)]
#[command(
    after_long_help = r#"The tool expects a Hexmake file to exist in the current directory.
A Hexmake file looks like this:

```json
{
  "rules": [
    {
      "name": "main",
      "inputs": [
        "main.c",
      ],
      "outputs": [
        "out/main"
      ],
      "commands": [
        "cc -o out/main main.c"
      ]
    }
  ]
}
```
"#
)]
pub struct Args {
    /// The rules or output files to build
    pub targets: Vec<Arc<String>>,

    /// List available targets and exit
    #[arg(long)]
    pub list_targets: bool,
}
