# Hexmake

Hexmake is a modern Make-like tool designed by [Lex Spoon][lex-spoon]. It improves on classic
Make in a few ways:

* It uses content-based caching rather than time stamps. A cached artifact will
  only be used if it has the same input files, the same list of command lines,
  and the same environment variables.
* Build outputs are cached in a side directory and can be pulled from further back
  in time than the most immediate build. For example, if you run a build,
  change a file, build again, and then revert your change, then the next build will
  all go to the cache. Likewise, if you change Git branches, build, and then change
  back, you will not have to rebuild anything back on your main branch.
* Build commands are run in a sandbox rather than directly in the source checkout.
  That way, if your command depends on something you did not declare, the build
  will fail.

The tool is general minimal. It does one job well---
dependency-driven rebuilds with caching---and then leaves it to you to write
a script around it based on the needs of your project. Some examples are given in the `examples` directory that you can start from.

## Installation

At the time of writing, the way to install Hexmake is to check out the
Git repository, [install Rust], and then run `./scripts/install`.

The plan is to eventually support `cargo install hexmake` as a shorter process for those who want to use Hexmake but not to contribute to it.

## Minimal example

Here is a simple example for building a small C program.

You need to make a file named `Hexmake` that has the build instructions for the
project. Here is a basic one:
```json
{
  "rules": [
    {
      "name": "lib.o",
      "inputs": [
        "lib.c",
        "lib.h"
      ],
      "outputs": [
        "out/lib.o"
      ],
      "commands": [
        "gcc -o out/lib.o -c lib.c"
      ]
    },
    {
      "name": "main.o",
      "inputs": [
        "lib.h",
        "main.c"
      ],
      "outputs": [
        "out/main.o"
      ],
      "commands": [
        "gcc -o out/main.o -c main.c"
      ]
    },
    {
      "name": "main",
      "inputs": [
        "out/lib.o",
        "out/main.o"
      ],
      "outputs": [
        "out/main"
      ],
      "commands": [
        "gcc -o out/main out/lib.o out/main.o",
        "chmod 755 out/main"
      ]
    }
  ]
}
```

There are three rules in this Hexmake file. From the top to the bottom:

* The rule named `lib.o` takes `lib.c` and `lib.h` as input, and it generates a
  file `out/lib.o` as output.
* The rule named `main.o` does the same thing, but for `main.c`.
* The rule `main` takes the previous two outputs and links them into `out/main`.

Save that file as `Hexmake`, and create the three input files.
* `lib.h`:
  ```c
  int sum(int a, int b);
  ```
* `lib.c`:
  ```c
  #include "lib.h"

  int sum(int a, int b) 
  {
    return a+b;
  }
  ```
* `main.c`
  ```c
  #include "lib.h"

  #include <stdio.h>
  #include <stdlib.h>

  int main(int argc, char **argv) 
  {
    if (argc != 3) 
      {
        printf("Usage: sum a b\n");
        return 1;
      }


    int a = atoi(argv[1]);
    int b = atoi(argv[2]);
    int answer = sum(a, b);

    printf("Sum: %d\n", answer);

    return 0;
  }
  ```

You can now build the program by running `hexmake main`.

Experiment with different changes to the input files and/or the Hexmake file
yourself. For example, if you change `lib.c` and rebuild, the tool will reuse
its build of `main.o`. If you change `lib.h`, it will rebuild everything. If you
change a command line in the `Hexmake` file, it will rebuild that rule even if
its input files have not changed.

Touching a file will, by itself, not cause a rebuild. The timestamps are
irrelevant for Hexmake.

## Hexmake invocation
The invocation of Hexmake looks like this:
```
hexmake target...
```

The command will read the Hexmake file and
then attempt to build the given list of targets.

At least one target must be supplied.

A target can be in one of two forms:

* It can be the name of any rule in the Hexmake file.
* It can be an output file, in which case it must start with `out/`.

## Exit codes
Hexmake returns the following exit codes:

* 0\. The command ran successfully.
* 1\. The invocation was wrong in some way, e.g. a bad Hexmake file or
  a bad command-line argument.
* 2\. A build error occurred.


## Concepts

Hexmake is an [artifact-based build tool][artifact-based].
Hexmake consumes and builds
**artifacts**, where artifacts can be either supplied from source code
or build from the tool itself.

The first kind of artifact is a **source tree**.
These are checked into your source code as a file or as a directory tree.
You refer to them using a relative path from the root of the repository to the
file or directory that is the root of the tree. Every file underneath the specified
path will be included as part of the tree.

The other kind of artifact is an **output artifact**. This is
a single file that is produced by a build rule.

Outputs always go into a directory named `out`, and source trees
are always located outside of `out`. This strict segregation is important
Hexmake to maintain an accurate build cache for you. It needs to know
that build commands always map input files to output files and never
go backward to update the input files.

Build artifacts are combined together via **rules**. A rule has
the following components:

* A list of input artifacts. These artifacts can be both source
  trees and the outputs of other build rules.
* A list of outputs. These must all start with `out/`, and they
  must all be an individual file.
* A list of commands. These are shell-script commands and will
  be run in the order that they are listed.
* A name. The name of a rule is used as a short-hand for
  specifying requests to the tool as well as for the tool
  to give feedback to the user.


## Hexmake file reference

A `Hexmake` file is a JSON file that matches
the following TypeScript types.

```typescript
type HexmakeFile = {
  rules: Rule[]
}

type Rule = {
  name: RuleName
  outputs: OutputArtifact[]
  inputs: Artifact[]
  commands: string
}

type RuleName = string
type Artifact = OutputArtifact | SourceTree
type OutputArtifact = string
type SourceTree = string
```

### Artifact

```typescript
type Artifact = OutputArtifact | SourceTree
```

An Artifact is either an OutputArtifact or a SourceTree.
These are the two kinds of permissible inputs to
a build rule.

### HexmakeFile

```typescript
type HexmakeFile = {
  rules: Rule[]
}
```
A Hexmake file is a JSON file that has a list of rules in it.

### OutputArtifact

```typescript
type OutputArtifact = string
```

An OutputArtifact is a string that is formatted as
a filesystem path starting with `out/`. 
An example is `"out/c/main.o"`.

### Rule

```typescript
type Rule = {
  name: RuleName
  inputs: Artifact[]
  outputs: OutputArtifact[]
  commands: string
}
```

A Rule in a Hexmake file tells the tool how to build an output out of 

### RuleName

```typescript
type RuleName = string
```

A rule name is a string that gives a name to a rule.
Typical examples would be `"main.o"` `"test/quick"`.

Rule names are used for output from the tool and on the command
line. On the command line, they give a short way to request
a file to be built, e.g. `hexmake main.o` rather than `hexmake out/c/main.o`.

A rule should follow the rules of a source tree: it is a sequence of filenames,
separated by the slash character, and the first component cannot be `out`.


### SourceTree

```typescript
type SourceTree = string
```

A SourceTree is a string that is formatted as
a filesystem path but that does not begin with `out/`.
It refers to an input file or file tree.

If the build rule for a source tree runs, then it is
required that the source tree refers to an actual file
or directory tree in the original inputs. However, it
is permitted for a source tree to not exist if the
associated build rule never runs.
