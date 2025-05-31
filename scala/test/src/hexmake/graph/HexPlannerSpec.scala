package hexmake.graph

import hexmake.ast.{HexRule, HexmakeFile}
import hexmake.graph.{HexPlanner, TaskNode}
import hexmake.util.HexPaths.*
import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers

import scala.collection.immutable.ArraySeq

class HexPlannerSpec extends AnyFunSuite with Matchers:
  test("assemble a task graph") {
    val hexmakeFile = HexmakeFile(
      ArraySeq.empty,
      ArraySeq(
        HexRule(
          hexPaths("out/foo"),
          hexPaths("out/foo.o"),
          ArraySeq("gcc -o out/foo out/foo.o")
        ),
        HexRule(
          hexPaths("out/foo.o"),
          hexPaths("foo.c"),
          ArraySeq("gcc -o out/foo.o foo.c")
        ),
        HexRule(
          hexPaths("out/bar"),
          hexPaths("out/bar.o"),
          ArraySeq("gcc -o out/bar out/bar.o")
        ),
        HexRule(
          hexPaths("out/bar.o"),
          hexPaths("bar.c"),
          ArraySeq("gcc -o out/bar.o bar.c")
        )
      )
    )

    val taskNodes: ArraySeq[TaskNode[HexRule]] =
      assembleTasks(hexmakeFile, "out/foo", "out/bar")

    taskSummary(taskNodes) shouldBe
      """Task: out/foo
          |  Depends on tasks: out/foo.o
          |Task: out/foo.o
          |  Used by tasks: out/foo
          |Task: out/bar
          |  Depends on tasks: out/bar.o
          |Task: out/bar.o
          |  Used by tasks: out/bar
          |""".stripMargin

    checkTasks(taskNodes)
  }

  test("reuse tasks already made") {
    val hexmakeFile = HexmakeFile(
      ArraySeq.empty,
      ArraySeq(
        HexRule(
          hexPaths("out/foo"),
          hexPaths("out/foo.o"),
          ArraySeq("gcc -o out/foo out/foo.o")
        ),
        HexRule(
          hexPaths("out/foo.o"),
          hexPaths("foo.c"),
          ArraySeq("gcc -o out/foo.o foo.c")
        ),
        HexRule(
          hexPaths("out/bar"),
          hexPaths("out/bar.o"),
          ArraySeq("gcc -o out/bar out/bar.o")
        ),
        HexRule(
          hexPaths("out/bar.o"),
          hexPaths("bar.c"),
          ArraySeq("gcc -o out/bar.o bar.c")
        )
      )
    )

    // foo.o and then foo
    val taskNodes: ArraySeq[TaskNode[HexRule]] =
      assembleTasks(hexmakeFile, "out/foo.o", "out/foo")

    taskSummary(taskNodes) shouldBe
      """|Task: out/foo.o
           |  Used by tasks: out/foo
           |Task: out/foo
           |  Depends on tasks: out/foo.o
           |""".stripMargin
    checkTasks(taskNodes)

  }

  test("skip top-level tasks already made") {
    val hexmakeFile = HexmakeFile(
      ArraySeq.empty,
      ArraySeq(
        HexRule(
          hexPaths("out/foo"),
          hexPaths("out/foo.o"),
          ArraySeq("gcc -o out/foo out/foo.o")
        ),
        HexRule(
          hexPaths("out/foo.o"),
          hexPaths("foo.c"),
          ArraySeq("gcc -o out/foo.o foo.c")
        ),
        HexRule(
          hexPaths("out/bar"),
          hexPaths("out/bar.o"),
          ArraySeq("gcc -o out/bar out/bar.o")
        ),
        HexRule(
          hexPaths("out/bar.o"),
          hexPaths("bar.c"),
          ArraySeq("gcc -o out/bar.o bar.c")
        )
      )
    )

    // foo and then foo.o
    val tasks: ArraySeq[TaskNode[HexRule]] =
      assembleTasks(hexmakeFile, "out/foo", "out/foo.o")

    taskSummary(tasks) shouldBe
      """|Task: out/foo
           |  Depends on tasks: out/foo.o
           |Task: out/foo.o
           |  Used by tasks: out/foo
           |""".stripMargin
    checkTasks(tasks)
  }

  test("build a task once to get multiple outputs") {
    val hexmakeFile = HexmakeFile(
      ArraySeq.empty,
      ArraySeq(
        HexRule(
          hexPaths("out/foo"),
          hexPaths("out/foo.c", "out/bar.c"),
          ArraySeq("gcc -o out/foo out/foo.c", "out/bar.c")
        ),
        HexRule(
          hexPaths("out/foo.c", "out/bar.c"),
          hexPaths("gensources"),
          ArraySeq("./gensources")
        )
      )
    )

    val taskNodes: ArraySeq[TaskNode[HexRule]] =
      assembleTasks(hexmakeFile, "out/foo")

    taskSummary(taskNodes) shouldBe
      """|Task: out/foo
           |  Depends on tasks: out/foo.c
           |Task: out/foo.c, out/bar.c
           |  Used by tasks: out/foo
           |""".stripMargin
    checkTasks(taskNodes)
  }

  def assembleTasks(
      hexmakeFile: HexmakeFile,
      outputs: String*): ArraySeq[TaskNode[HexRule]] =
    new HexPlanner(hexmakeFile).assembleTasks(
      ArraySeq.from(hexPaths(outputs: _*)))

  def taskSummary(taskNodes: ArraySeq[TaskNode[HexRule]]): String =
    val result = new StringBuilder
    for (taskNode <- taskNodes) {
      result ++= s"Task: ${taskNode.task.outputs.mkString(", ")}\n"
      if (taskNode.dependsOn.nonEmpty) {
        result ++= s"  Depends on tasks: ${taskNode.dependsOn.map(t => t.task.outputs.head).mkString(", ")}\n"
      }
      if (taskNode.usedBy.nonEmpty) {
        result ++= s"  Used by tasks: ${taskNode.usedBy.map(t => t.task.outputs.head).mkString(", ")}\n"
      }
    }
    result.toString

  /** Internal consistency checks for a list of tasks */
  def checkTasks(taskNodes: ArraySeq[TaskNode[HexRule]]): Unit =
    for (taskNode <- taskNodes)
      for (dep <- taskNode.dependsOn)
        taskNodes should contain(dep)
        dep.usedBy should contain(taskNode)
      for (usedBy <- taskNode.usedBy)
        taskNodes should contain(usedBy)
        usedBy.dependsOn should contain(taskNode)
