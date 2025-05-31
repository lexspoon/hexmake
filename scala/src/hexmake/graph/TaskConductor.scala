package hexmake.graph

import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.{LinkedBlockingQueue, Semaphore}
import scala.collection.immutable.ArraySeq

/** A generic task conductor. It tracks a set of [[TaskNode]]s and can run them
  * in dependency order, on multiple threads.
  */
class TaskConductor[T >: Null](runTask: T => Boolean, concurrency: Int):
  /** A semaphore for monitoring task completion. It is given a permit for each
    * time that a task completes.
    */
  private val completionSempahore = new Semaphore(0)

  /** Whether any of the tasks failed */
  private val anyTaskFailed = new AtomicBoolean

  /** A queue of tasks that are ready to execute */
  private val tasksReady = new LinkedBlockingQueue[TaskNode[T]]

  /** A sentinel task that indicates that this conductor is shutting down */
  private object ShutDownTask extends TaskNode[T](null)

  /** Run the given set of tasks, being careful to only run a task after all of
    * its dependencies are ready. Return whether all of the tasks succeeded or
    * not.
    */
  def runTasks(taskNodes: ArraySeq[TaskNode[T]]): Boolean =
    addInitialTasks(taskNodes)
    startWorkers()
    waitForAllTasks(taskNodes.size)
    tasksReady.put(ShutDownTask)
    !anyTaskFailed.get

  /** Add initial tasks that are already ready to run */
  private def addInitialTasks(taskNodes: ArraySeq[TaskNode[T]]): Unit =
    for task <- taskNodes.filter(_.dependsOn.isEmpty)
    do tasksReady.put(task)

  /** Start workers running */
  private def startWorkers(): Unit =
    for _ <- 1 to concurrency
    do new Thread(() => doWork()).start()

  /** Wait until either all tasks completed successfully or any task failed */
  private def waitForAllTasks(numTasks: Int): Unit =
    var tasksLeft = numTasks
    while tasksLeft > 0
    do
      completionSempahore.acquire()
      if anyTaskFailed.get then return
      tasksLeft = tasksLeft - 1

  /** The work that one worker thread should do. It will run tasks from the
    * ready queue until there are no more tasks.
    */
  private def doWork(): Unit =
    while true do
      val task = tasksReady.take()
      if task == ShutDownTask then
        // The conductor is shutting down.
        // Put the task back, for other workers to see it,
        // and then exit.
        tasksReady.put(task)
        return

      val taskSucceeded =
        try runTask(task.task)
        catch
          case err: Throwable =>
            err.printStackTrace()
            false

      if !taskSucceeded then
        // The task failed. Start shutting everything down.
        anyTaskFailed.set(true)
        tasksReady.put(ShutDownTask)
        completionSempahore.release()
        return

      completionSempahore.release()
      for otherTask <- task.usedBy do
        if otherTask.dependencyFinished() == 0
        then tasksReady.put(otherTask)
