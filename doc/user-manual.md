# Hexmake

Hexmake is a modern Make-like tool designed by Lex Spoon. It improves on classic
Make in a few ways:

* It uses content-based caching rather than time stamps. A cached artifact will
  only be used if it has the same input files, the same list of command lines,
  and the same environment variables. Otherwise, the 
* Build commands are run in a sandbox in order to decrease the likelihood of an
  accidental missed dependency.

The tool has minimal niceties for larger project. It can be used directly for a
small project with 5-10 dependencies, but for a more complex project, you should
plan to use a wrapper script that you write in Python or Ruby. The reason for
this setup is that it seems hard to beat a dedicated scripting language like
Python or Ruby at its own game; instead of trying, Hexmake is designed to
combine well with these other tools.

## Installation

Once v1.0 is released, the plan is that you can install the tool with `cargo
install hexmake`.

However, at the time of writing, you need to check out the Git repository and
then run `./scripts/install`.

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

You can now build the program by running `hexmake out/main`.

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

Hexmake is an artifact-based build tool. Hexmake consumes and builds
**artifacts**, where artifacts can be either supplied from source code
or build from the tool itself.

A source code artifact is specified as the root of a tree of inputs.
You can specify either an individual file name or a directory, and if
you specify a directory, it will indicate the entire tree under that
directory.

Build outputs are always individual files. Whenever you need to build
a tree of files, you should construct a zip file of them rather than
pass the tree around as is.

Build outputs always go under the `out` directory. The tool assumes
in multiple places that if a string starts with `out`, then it's
a build artifact, and that otherwise it must be something else---either
a rule name or a source tree, depending on context.

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


## Hexmake reference

TODO

The `hexmake` tool looks for a file named `Hexmake` in the directory
that it is invoked. This file has instructions for the available build
targets 


## Comparisons to other tools

Here is why I build Hexmake after trying everything else that is out there.

* Bazel is inspiring, but it is hard to use in practice without
  having something like the Google build team to support you. What
  makes it so hard is the way that you extend and configure it
  with Starlark modules. They are provided off the shelf and hard
  to modify for your own use. They often do 100 things you don't need
  and yet not quite what you need for your own environment. Based on
  that experiment, I think it's better for people to write their own
  script around the build tool and simply check the script into
  their repository. I find with Bazel that many problems would be simple
  if I could edit the script file but that it's impractical to do so
  given how everything is stitched together.
* Nix is also inspiring. It is an artifact-based build tool that
  is minimal and gets out of the way.
  * It does not give you a good way to name a top-level list of artifacts
    and have them all depend on each other. I think it is interesting
    that a "build tool" has a list of "build artifacts" it manages,
    each with a simple top-level name.
  * It has an extensive scripting language that is still not competitive
    with Ruby or Python for practical usage. I already thought it was
    a questionable direction to design a custom scripting language
  * It is minimal in the wrong ways. A build still has a notion of
    targets.
  * It encourages a style of building dependencies internally rather
    than using system dependencies. For things like `gcc` or `firefox`,
    it seems much better to me to have the developer manage them and
    then to invoke them via the PATH. This approach matches the general
    idea that a developer will install their dependencies and then,
    far from wanting their tools to be isolated from each other, to
    rely on each other for their individual jobs. When a developer
    install a tool at `/usr/bin/gcc`, or elsewhere on their PATH,
    then that is the version of `gcc` that they want all C compilation
    in their experience to use.
* Mill is very effective for a standard Scala project, but I found it
  challenging with Mill to just write random rules for non-Scala build
  steps that are not built in. In general, I often what a build tool
  that isn't specific to one language but rather---like Make---lets
  me declare my dependencies and give a command line for building
  each artifact.


## Background reading

[Artifact-Based Build Systems][artifact-based]. This page from the Bazel
documentation explains the general idea of an artifact-based build tool.

[Recursive Make Considered Harmful][recursive-make]. This article from Peter
Miller reviews how Make works and how to get the best results from it. The
sandboxing in Hexmake is designed to steer users away from the traps that Miller
identified.

[Lambda: The Ultimate Little Language][ultimate-little-language], by Olin
Shivers. Olin argues that when you want a little language for something, then
you should reuse as much as possible from some existing general-purpose language
rather than inventing things like variables, functions, and arrays from scratch
and probably doing it badly. Hexmake follows this approach by planning to be
invoked via a wrapper script written in a dedicated scripting language. Olin
gives examples of embedding languages into Scheme, and while that often works
well, I think for Hexmake it is better for people to use whatever scripting
language already makes the most sense for their project.

[Nix Reference Manual][nix-reference]. The introduction includes a rationale and
explanation of how Nix works. I differ on very many specific design decisions,
but it is well worth reading since Nix is so carefully designed and built.

[The Mill Build Tool][mill]. The web page for Mill has many helpful design notes
about how it works and why.

[artifact-based]: https://bazel.build/basics/artifact-based-builds
[mill]: https://mill-build.org/mill/index.html
[nix-reference]: https://nix.dev/manual/nix/2.28/
[recursive-make]: https://aegis.sourceforge.net/auug97.pdf
[ultimate-little-language]: https://3e8.org/pub/scheme/doc/Universal%20Scripting%20Framework%20(Lambda%20as%20little%20language).pdf

