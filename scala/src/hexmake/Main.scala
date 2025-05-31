package hexmake

import hexmake.ast.*
import hexmake.exec.{BuildDirManager, RunHexRule}
import hexmake.graph.{HexPlanner, TaskConductor, TaskNode}

import java.io.File
import scala.collection.immutable.ArraySeq

object Main:
  def main(args: Array[String]): Unit =
    if (args.isEmpty) then
      System.err.println("Usage: hexmake target [target...]")
      sys.exit(2)

    val file = new File("Hexmake")
    if !file.isFile then
      System.err.println("Hexmake file does not exist")
      sys.exit(2)

    val hexmakeFile: HexmakeFile = HexmakeParser.parseFile(file)

    val tasks: ArraySeq[TaskNode[HexRule]] =
      new HexPlanner(hexmakeFile).assembleTasks(
        ArraySeq.from(args).map(p => HexPath(p)))

    val buildDirManager = new BuildDirManager
    buildDirManager.clean()

    val taskConductor = new TaskConductor(
      new RunHexRule(buildDirManager),
      HexSettings().concurrency)

    val succeeded = taskConductor.runTasks(tasks)

    if (!succeeded) sys.exit(1)
