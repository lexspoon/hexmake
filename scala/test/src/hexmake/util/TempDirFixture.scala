package hexmake.util

import hexmake.util.FileUtils.*
import org.scalatest.{BeforeAndAfterEach, Suite}

import java.io.File
import java.nio.file.Files

/** This fixture creates a temporary directory for a test to run in.
  */
trait TempDirFixture extends BeforeAndAfterEach:
  self: Suite =>

  protected var tempDir: File = _

  override def beforeEach(): Unit =
    super.beforeEach()
    tempDir = Files.createTempDirectory(getClass.getName).toFile

  override def afterEach(): Unit =
    recursiveDelete(tempDir)
    tempDir = null
