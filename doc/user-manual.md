=======
Hexmake
=======

(Scope)

Minimal example
===============

Concepts
========

Hexmake invocation
==================
The invocation of Hexmake looks like this:
```
hexmake target...
```
The command will read the Hexmake file and
then attempt to build the given list of targets.

At least one target must be supplied.

Exit codes
==========
Hexmake returns the following exit codes:

* 0\. The command ran successfully.
* 1\. A build error occurred.
* 2\. The invocation was wrong in some way, e.g. a bad Hexmake file or
  a bad command-line argument.

Hexmake reference
=================

Overall Hexmake file
--------------------

The `hexmake` tool looks for a file named `Hexmake` in the directory
that it is invoked. This file has instructions for the available build
targets 
