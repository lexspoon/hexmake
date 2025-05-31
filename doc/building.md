This file has instructions for building Hexmake.

Environment setup
-----------------
Optionally, install the prequisites of Scala Native, using
[its own setup instructions](https://scala-native.org/en/stable/user/setup.html).
You can also develop on the Java VM version of the tool, though,
so long as you find a way to test the changes with
Scala Native.

Building the tool
-----------------
If you don't already have Mill installed, then use the "mill" script that
is checked into this repository. It will download Mill for you and then
run it. You can either run it in place, or copy the script to a directory that
is on your PATH.

Build a native executable with `mill native.nativeLink`. This is the recommended
way to use and redistribute the tool in most circumstances.
The resulting executable is located at `out/native/nativeLink.dest/out`.

Build a Java assembly with `mill jvm.assembly`. This way is usable
in scenarios where a native executable is inconvenient for some reason.
The resulting assembly is at `out/jvm/assembly.dest/out.jar`.

Both of these can be run together by running `mill build`.

Running tests
-------------
Run tests by using `mill test`. This will run the tests with the JVM implementation.

To test the native build, run `mill native.test`. This is much slower, but it's
good to make sure that it works.

Importing into IntelliJ
-----------------------
Install IntelliJ using [its own installation instructions](https://www.jetbrains.com/help/idea/installation-guide.html).
The Community Edition is adequate for developing this software, but if you use IntelliJ a lot,
consider purchasing the Ultimate edition.

At the command line, run the following:

```
mill mill.idea.GenIdea/idea
```

This will populate a .idea directory for you. You can then import the project into
IntelliJ.
