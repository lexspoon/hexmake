Hexmake
=======

Hexmake is a small build tool that provides good caching behavior
and then gets out of the way. It is organized like Make, but
with a variety of small improvements:

* Process whole directory trees as inputs, not just individual files.
* Support .gitignore when doing so.
* When deciding whether to rebuild, use the contents of the files
  rather than their timestamps.
* Also include the command line and environment variables as inputs.
* Always put build outputs into an `out` directory rather than
  ever putting them back in the source trees.
* Use your favorite scripting language for templating
  rather the tool having its own.

Compared to Nix, which is otherwise a masterpiece:

* Do not recompile Firefox from source code. Use the system libraries and tools
  the way they already exist on the system rather than control them with the
  build tool. The idea is that a developer gets a better experience when their
  different tools all integrate with each other. Bringing that idea to the build
  system, it is preferable if the build system uses the same compilers and other
  tools that the developer is already using, and likewise, it is better if the
  outputs of the build system are ready to be used by the developer and do not
  need some special side environment to be run.

I hope you enjoy Hexmake! Tell me about your experiments if you try it out.
