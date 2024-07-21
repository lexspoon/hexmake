This file has instructions for building Hexmake.

Environment setup
-----------------
Install Scala Native, using
[its own setup instructions](https://scala-native.org/en/stable/user/setup.html).


Building an executable
----------------------
Run `sbt nativeLink` to generate an executable. The resulting binary
will be located at `target/scala-3.3.3/native/Main`.

To generate an optimized build, use `sbt nativeLinkReleaseFull`.
This version of the build is slower but will produce a better
resulting binary file.


Running tests
-------------
You can run tests at the command line by using `sbt test`.


Importing into IntelliJ
-----------------------
Install IntelliJ using [its own installation instructions](https://www.jetbrains.com/help/idea/installation-guide.html).
The Community Edition is adequate for developing this software, but if you use IntelliJ a lot,
consider purchasing the Ultimate edition.

Within IntelliJ, you will want to install the Scala plugin. After that, you can
import this Git repository as an IntelliJ project. Specify that it is an "SBT project"
when you import. IntelliJ will then scan the `build.sbt` file to figure out everything
else it needs to know.
