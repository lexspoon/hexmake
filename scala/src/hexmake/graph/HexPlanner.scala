package hexmake.graph

import hexmake.ast.{HexPath, HexRule, HexmakeFile}

import scala.collection.immutable.ArraySeq
import scala.collection.mutable
import scala.collection.mutable.LinkedHashMap

/** Plan a graph of rule executions, including their dependencies with each
  * other.
  */
class HexPlanner(hexFile: HexmakeFile):
  private val rulesByOutput: Map[HexPath, HexRule] =
    hexFile.rules.flatMap(r => r.outputs.map(o => (o, r))).toMap

  private val taskForRule: LinkedHashMap[HexRule, TaskNode[HexRule]] =
    LinkedHashMap.empty

  /** Find all tasks needed to build the given outputs */
  def assembleTasks(
      outputs: ArraySeq[HexPath]): ArraySeq[TaskNode[HexRule]] =
    for (output <- outputs) createTasks(output)
    ArraySeq.from(taskForRule.values)

  /** Create any new tasks needed to build the given output. This will also
    * recursively create tasks for all inputs needed to build this output.
    */
  private def createTasks(output: HexPath): Option[TaskNode[HexRule]] =
    if !output.isOutput then
      // It's an original input from the source tree. There's nothing
      // to build.
      return None
    val rule = rulesByOutput(output)
    if taskForRule.contains(rule) then
      // There's already a task for this rule. Return it.
      return Some(taskForRule(rule))

    // Make a new task
    val task = new TaskNode(rule)
    taskForRule(rule) = task
    for (input <- rule.inputs)
      createTasks(input).foreach(t => task.addDependsOn(t))
    Some(task)
