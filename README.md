Hexmake runs a multi-step build using caching. You give it a file describing all
the possible build steps along with their inputs and outputs. The tool will then
chain them together to produce an output, use cached results from prior builds
when possible.

The tool does not handle other steps of a build and is designed to fit into the
build ecosystem you already have. You are in the sweet spot for this tool if
either of these is true:

* You started out using shell scripts for running builds and tests, but your
  project has become complicated enough that builds take multiple steps and run
  slowly. You would like to stop building the world all the time and switch to
  only rebuilding things when their inputs have changed.
* You have implemented Bazel, Pants, or Buck, but you are finding that you
  regularly spend an hour or more fighting the tool to get it to do something
  you want. Hexmake is less opinionated and more general-purpose than these
  tools and is a fit whenever you would prefer to write a little script code of
  your own rather than  file tickets with a plugin author.

Hexmake is inspired by Make and comes with some newer features:

* Process whole directory trees as inputs, not just individual files.
* Support .gitignore files so that builds will only depend on
  checked-in source code.
* When deciding whether to rebuild, use the contents of the files
  rather than their timestamps.
* Also treat a change in command line or a change in environment
  variables as grounds for rebuilding.
* Consistently put all build outputs into an `out` directory rather than back
  into the source tree.
* Run builds in a separate sandbox directory. That way, if you depend on an
  input you did not declare, it will usually cause a build break.

Hexmake leaves some things out on purpose:
* It provides no way to build and host major development tools such as
  Firefox or Docker. You are expected to install those separately.
* It has no built-in variables or control structures. Instead,
  you should wrap Hexmake with a script in your own preferred
  scripting language such as Python, Ruby, or Node.
