## Comparisons to other tools

Here is why I built Hexmake after trying everything else that is out there.

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


## Why the tool is so sparse

Based on the arguments in
[Lambda: the Ultimate Little Language][ultimate-little-language],
Hexmake has absolutely no scripting support in its own file format. It doesn't
have variables, conditional inclusion, templates or modules. Instead, you should
write those in your own favorite scripting language.

A related experience for me was to make a large change to Bison, which uses the
M4 templating language. I did [a deep dive on M4][lex-m4] and tried to
understand how it can start out so simple but become such a nightmare when you
use it as much as Bison does. I came to the conclusion that for larger-scale
templates, you want things to work much the same as with the functions,
variables, and modules of a general-purpose language. So, for Bison, I think it
would be more maintainable if it used C, its host language, to generate
templates.

For Hexmake, the plan is that you use a scripting language like Python, Ruby, or
JavaScript and emit the JSON configuration file as an output. You then get to
use high quality variables, conditions, subroutines, and modules without having
to learn and work with a new scripting language.

## Adding rather than removing

When I reflect on times where I found Bazel difficult to customize, it is
because I needed to subtract and modify from something the tool gives rather
than add.

For example, I wanted to set up a plugin with Scala in my build, but the way
Bazel works is that you invoke `scala_library` and it looks up a "toolchain"
behind the scenes that does what you need. I had trouble getting a plugin
configured for all users of `scala_library` and spent hours working on it. Based
on [this Bazel ticket][bazel-issue-scala-plugin], others have lost similar large
amounts of time.

My workaround that I posted on that ticket is one that I think works more
broadly and is a very reasonable way for things to just work to begin with.
Instead of having `scala_library` be so magical, write your own `scala_library`
routine that expands out to a lower-level invocation of Scala with the correct
parameters. Taking this idea further, go ahead and use the CLI interface as the
hand-off between the build system and the specific compilers and other tools it
runs; so, have your `scala_library` expand into an actual `scalac` command line
with all of the parameters spelled out. This is very easy for  someone to script
for themselves and can only be made worse by adding a layer to automate it.

More broadly than that specific example, Bazel is controlled by extension files
written in Starlark that are then patched into your build. Simple builds often
depend on thousands of lines of Starlark downloaded from dozens of authors. On a
good day, this all works and does what you need with no fuss. However, the very
first time you want anything different, you have to go decipher and debug that
large amount of configuration code. If you are lucky, you will find a parameter
already that can be specified to change the behavior how you want. If you are
less lucky, you need to modify the Starlark or to not use it, but then you have
to figure out how to branch the Git repo the Starlark is in, depend on it in
your code, and get your change into it. Oh, and you also have to make the
change, so you have to understand all the existing code that is in the extension
right now.

If you do this even once for a given project, you've blown away the time on that
one problem that you could have spent on crafting your own script code to
generate things in the right way for your project. For a typical project of up
to about ten people, I believe you can set up your build system in just a few
hours if you script everything manually. Using an extension will reduce these
few hours to maybe half an hour when things go well, but in exchange, they will
lose you many hours of time the first time you need to change how any of it
works.

As a positive experience, I have had good experience with writing web servers
using [the Rouille framework][rouille]. With Rouille, you start with a very simple
function that accepts a request and calculates a response from it, like this:
```rust
use rouille::Request;
use rouille::Response;

rouille::start_server("0.0.0.0:80", move |request| {
    Response::text("hello world")
});
```

You can then augment this with CORS, error handling, authorization, and many
other things, gradually and piecemeal, as you encounter a desire for them. To
contrast, most web frameworks have all of those things built in, but then to use
them for your project, you have to do a similar debugging and modification
process as I discussed above for Bazel extensions. You can easily lose many
hours of time if you need a behavior that is not the default and does not have a
parameter for it.

Putting all this together, it's much better when a tool and its libraries are
additive. It is only a little bit of trouble to combine multiple low-level
utilities and to write script-code goo to put them together in the right way. It
is a much larger trouble if you need to unpack your third-party components and
modify them in some way.

## Background reading

[Artifact-Based Build Systems][artifact-based]. This page from the Bazel
documentation explains the general idea of an artifact-based build tool.

[Recursive Make Considered Harmful][recursive-make]. This article from Peter
Miller reviews how Make works and how to get the best results from it. The
sandboxing in Hexmake is designed to steer users away from the traps that Miller
identified.

[Lambda: The Ultimate Little Language][ultimate-little-language], by my
advisor Olin Shivers. Olin argues that when you want a little language for something, then
you should reuse the general-purpose parts of an existing language rather
than reinvent things like variables and function calls; these are more subtle
than they look and are often implemented badly when bolted onto a file format.
For this reason, Hexmake does not have its own subroutines or variables.
You should use a scripting language like Python, generate a `Hexmake` file,
and then run `hexmake`.

[Nix Reference Manual][nix-reference]. The introduction includes a rationale and
explanation of how Nix works. Nix is carefully designed and built, and there is
a lot of practical experience with it at this point.

[The Mill Build Tool][mill]. The web page for Mill has many helpful design notes
about how it works and why.

[artifact-based]: https://bazel.build/basics/artifact-based-builds
[lex-m4]: https://blog.lexspoon.org/2023/06/why-m4-macro-syntax-goes-so-wrong.html
[lex-spoon]: https://lexspoon.org
[mill]: https://mill-build.org/mill/index.html
[nix-reference]: https://nix.dev/manual/nix/2.28/
[relax-json]: https://github.com/eteeselink/relax-json
[recursive-make]: https://aegis.sourceforge.net/auug97.pdf
[rouille]: https://docs.rs/rouille/latest/rouille/
[ultimate-little-language]: https://3e8.org/pub/scheme/doc/Universal%20Scripting%20Framework%20(Lambda%20as%20little%20language).pdf
