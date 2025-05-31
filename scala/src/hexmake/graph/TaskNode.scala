package hexmake.graph

import java.util.concurrent.atomic.AtomicInteger
import scala.collection.immutable.ArraySeq
import scala.collection.mutable

/** A generic task that can depend on other tasks to be run before it. */
class TaskNode[T](val task: T):
  /** Tasks this one depends on. */
  def dependsOn: ArraySeq[TaskNode[T]] =
    if _dependsOnImm eq null then _dependsOnImm = ArraySeq.from(_dependsOn)
    _dependsOnImm

  /** Tasks this one is used on. This is always an exact inverse of
    * [[dependsOn]].
    */
  def usedBy: ArraySeq[TaskNode[T]] =
    if _usedByImm eq null then _usedByImm = ArraySeq.from(_usedBy)
    _usedByImm

  /** Add a new dependency. */
  def addDependsOn(otherTask: TaskNode[T]): Unit =
    if !_dependsOn.add(otherTask) then return
    _dependsOnImm = null
    otherTask._usedBy += this
    otherTask._usedByImm = null
    _unbuiltDependencies.incrementAndGet

  /** Tell this task that one of its dependencies has finished building. The
    * return value is the number of task dependencies that still remain.
    */
  def dependencyFinished(): Int = _unbuiltDependencies.decrementAndGet

  /** Internal mutable version of [[dependsOn]] */
  private val _dependsOn = mutable.LinkedHashSet.empty[TaskNode[T]]

  /** Internal immutable version of [[dependsOn]]. */
  private var _dependsOnImm: ArraySeq[TaskNode[T]] = _

  /** Internal mutable version of [[usedBy]] */
  private val _usedBy = mutable.LinkedHashSet.empty[TaskNode[T]]

  /** Internal immutable version of [[dependsOn]]. */
  private var _usedByImm: ArraySeq[TaskNode[T]] = _

  /** A counter for the number of dependencies that still need to be built. */
  private val _unbuiltDependencies = new AtomicInteger
