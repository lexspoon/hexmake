package hexmake.exec

import hexmake.ast.HexPath
import hexmake.util.FileUtils.recursiveDelete

import java.io.File
import java.util.concurrent.atomic.AtomicInteger

/** Manage a build directory used for running commands */
class BuildDirManager:
  /** Delete the build directory */
  def clean(): Unit =
    recursiveDelete(buildDirParent)
    buildDirParent.mkdirs()

  /** Location to put temporary build directories */
  private def buildDirParent = new File(s"${HexPath.OUTPUT_ROOT}/.hex")

  /** Make a fresh build directory for running a rule.
    */
  def makeBuildDir(): File =
    val buildDirNum = buildDirsMade.getAndIncrement
    val result = new File(s"${HexPath.OUTPUT_ROOT}/.hex/build$buildDirNum")
    result.mkdirs()
    result

  private val buildDirsMade = new AtomicInteger
