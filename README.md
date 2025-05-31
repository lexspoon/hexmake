Hexmake
=======

Hexmake is the sparkly minimal build tool. It solves a
problem that is hard to write on your own--an artifact-based
build cache--while leaving the rest of your build
to be scripted by hand or through an outside mechanism.

The core functionality of Hexmake is similar to Bazel, Pants, or Buck.
Build artifacts are kept in a cache, where the cache is keyed
by all the inputs to a build step: the command, the environment
variables, and the hash code of all files used as inputs to the command.
Build commands can then be chained into further build commands,
in the style of Make.

To support interfacing with an external script, Hexmake is a CLI
tool that reads a JSON file of the full graph of build steps that
are available. The JSON files are reasonable to write by hand to get
started, but most larger projects will generate the file from
a Python script or other external framework.

The name Hexmake is chosen because it is a variant of the "Make"
tool, and because every other name based on "make" is taken
by some other project. Cmake, for caching make? Taken. Bcache,
for build cache? Taken. Bake? Forge? Cake? All taken.

Hexmake. Add a little sparkle and magic to your build, with Hexmake.
