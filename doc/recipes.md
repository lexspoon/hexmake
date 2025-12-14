Hexmake is minimal but supports many scenarios without needing a special feature
for it. This doc has recipes you can use to show how to do it.


Write a wrapper script in your favorite scripting language
==========================================================

You can start a simple project by writing a Hexmake file by hand. However, as
the project grows, you will start to have multiple rules that are similar to
each other but have some slight variation, e.g.  multiple .c files being
compiled, or multiple Rust crates being compiled.

At that point, what you can do is have a script named `build` in the root of
your repository. It can save a `Hexmake` file and then invoke `hexmake` as a
parameter.

Once you are writing your own script, you can make subroutines for common file
types. As well, when you generate build commands, you can generate them with the
parameters that make most sense for your environment.

A side benefit of this approach is that you can inspect the `Hexmake` file
manually. Your script may become complex, over time, but you can always inspect
its output and see the individual builds steps and how they work.


Debug a failed build step
=========================

If a build step fails, the engine will leave behind the build directory that it
used. You can cd to that directory and experiment until you figure out the exact
sequence of commands you would like to actually use.


Force tests to run again
========================

If you define test targets as normal build rules, then they will not run
themselves again once they succeed. This is normally a desirable behavior,
because Hexmake will skip any tests that are not affected by your most recent
changes.

```json
{
    "rules": [
        {
            "name": "test/rust",
            "outputs": [ "out/test/rust/ok" ],
            "inputs": [ "Cargo.toml", "src" ],
            "commands": [ "cargo test", "touch out/test/rust/ok" ]
        }
    ]
}
```

If you do want to force tests to run, though, for peace of mind or if you
suspect they depend on an external resource, you can adjust the commands to
include a timestamp in them. The following example adds `FORCE_TESTS=12345` to
the commands, and if the `12345` part is a timestamp, then Hexmake will have
never encountered this exact configuration and so will rerun the build rather
than use anything cached.
```json
{
    "rules": [
        {
            "name": "test/rust",
            "outputs": [ "out/test/rust/ok" ],
            "inputs": [ "Cargo.toml", "src" ],
            "commands": [
                "FORCE_TESTS=12345",
                "cargo test",
                "touch out/test/rust/ok"
            ]
        }
    ]
}
```

You can then adjust your top-level `build` script with a `--force-tests` option.
```javascript
config.rules.push({
    "name": "test/rust",
        "outputs": [ "out/test/rust/ok" ],
        "inputs": [ "Cargo.toml", "src" ],
        "commands": [
            ...(forceTests ? [`FORCE_TESTS=${Date.now()}`] : []),
            "cargo test",
            "touch out/test/rust/ok"
        ]
})
```
