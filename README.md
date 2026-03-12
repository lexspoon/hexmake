Hexmake runs a multi-step build using caching. You give it a file describing all
the possible build steps along with their inputs and outputs. The tool will then
chain them together to produce an output, using cached results from prior builds
when possible.

That's all the tool does, so you need to set up a larger build system around
it. It just gives you some functionality that is difficult to accomplish
using basic Unix tools on their own.

You are in the sweet spot for Hexmake if one of these is true:

* You started out using shell scripts for running builds and tests, but your
  project has become complicated enough that builds take multiple steps and run
  slowly. You would like to stop building the world all the time and switch to
  only rebuilding things when their inputs have changed.
* You have implemented Bazel, Pants, or Buck, but you are finding that you
  regularly spend an hour or more fighting the tool to get it to work
  right for your environment. You have thought to yourself, "I could easily
  write the script myself if only I knew where to put it".

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
* It also has no plugins, because plugins create a barrier where it
  is hard for you to adjust how they work. Instead, any smart behavior
  you need should be in your driver script.

See [the user manual](doc/user-manual.md) if you would like to try it out.
