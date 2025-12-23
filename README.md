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

* Use the system libraries and tools the way they already exist on the system
  rather than control them with the build tool. If your build depends on
  Firefox for running tests, then Hexmake will use the same Firefox that
  you are manually invoking as a developer.

I hope you enjoy Hexmake! Tell me about your experiments if you try it out.
