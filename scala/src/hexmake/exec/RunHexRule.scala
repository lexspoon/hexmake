package hexmake.exec

import hexmake.ast.{HexPath, HexRule}
import hexmake.util.FileUtils.recursiveDelete

import java.io.File
import java.nio.file.{Files, Path, StandardCopyOption}

/** A task runner for Hex rules */
class RunHexRule(buildDirManager: BuildDirManager) extends (HexRule => Boolean):

  /** Run one rule */
  override def apply(rule: HexRule): Boolean =
    val buildDir = buildDirManager.makeBuildDir()
    copyInputs(buildDir, rule)
    makeOutputParentDirs(buildDir, rule)
    try runCommands(buildDir, rule)
    catch case _: CommandFailedException => return false
    copyOutputs(buildDir, rule)
    true

  /** Copy all of the inputs of a rule into the given build directory.
    */
  private def copyInputs(buildDir: File, rule: HexRule): Unit =
    for (input <- rule.inputs) copyInput(buildDir, input)

  /** Copy one input into the given build directory.
    */
  private def copyInput(buildSubdir: File, input: HexPath): Unit =
    val inputFile = new File(input.path)
    if (!inputFile.exists) {
      throw new RuntimeException(s"Could not find input: $input")
    }

    val outputFile = new File(buildSubdir, input.path)

    if inputFile.isFile then
      // Copy one file
      outputFile.getParentFile.mkdirs()
      Files.copy(inputFile.toPath, outputFile.toPath)
      return

    // Copy a directory tree
    for (child <- inputFile.list())
      copyInput(outputFile, input.child(child))

  /** Make parent directories for each output of the given rule
    */
  private def makeOutputParentDirs(buildDir: File, rule: HexRule): Unit =
    val parentDirs =
      rule.outputs
        .map(out => out.path.substring(0, out.path.lastIndexOf('/')))
        .toSet
    for (parentDir <- parentDirs) new File(buildDir, parentDir).mkdirs()

  /** Run the commands for a build rule in the given build directory.
    */
  private def runCommands(buildDir: File, rule: HexRule): Unit =
    for (command <- rule.commands)
      println(command)
      val pb = new ProcessBuilder("bash", "-c", command)
      pb.directory(buildDir)
      pb.redirectError(ProcessBuilder.Redirect.INHERIT)
      pb.redirectOutput(ProcessBuilder.Redirect.INHERIT)
      val process = pb.start()
      val exitCode = process.waitFor()

      if exitCode != 0 then
        System.err.println(
          s"Build command exited with code $exitCode: $command")
        System.err.println(s"The build directory was $buildDir")
        throw new CommandFailedException

  private def copyOutputs(buildDir: File, rule: HexRule): Unit =
    for (outputPath <- rule.outputs)
      val output = outputPath.path
      recursiveDelete(new File(output))
      Files.copy(
        new File(buildDir, output).toPath,
        Path.of(output),
        StandardCopyOption.COPY_ATTRIBUTES
      )

class CommandFailedException extends RuntimeException
