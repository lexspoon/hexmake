package hexmake.util

import java.io.File

object TempFiles:
  /** Run code with a temp file with the given contents. The utility takes care
    * of creating and cleaning up the temp file.
    */
  def withTempFile[T](contents: String)(consumer: File => T): T =
    val tempFile = File.createTempFile("HexmakeParser", null)
    try consumer(tempFile)
    finally tempFile.delete()
