Hexmake
=======

Hexmake is Make but with some learnings from over the years:

* Treat whole directory trees as inputs, not just
  individual files.
* Treat the command line and certain environment
  variables as inputs.
* Separate the build outputs into a separate directory
  tree from the source code.
* Use your own favorite scripting language for templating,
  rather than having a crappy scripting language
  build into the tool.

Compared to Nix:

* Don't do things like recompile Firefox so that
  your build is the same as someone else's. It's better
  to let a developer set up their tools the way they like
  and then to have each tool trust the other ones to do
  what it is supposed to do.



I hope you enjoy! Tell me about your experiments if you try
it out.

Some background reading about all of this:

* [Recursive Make Considered Harmful](
    https://aegis.sourceforge.net/auug97.pdf),
  by Peter Miller.
  This kicked off my thinking about what you want in
  a build system.
* [Recursive Maven Consided Harmful](
      https://blog.lexspoon.org/2012/12/recursive-maven-considered-harmful.html),
  by Lex Spoon.
  This is my break down of the issue, a decade later.
