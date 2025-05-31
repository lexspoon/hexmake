package hexmake.util

import java.io.File

object FileUtils:
  def recursiveDelete(file: File): Unit =
    if (file.isFile)
      file.delete()
      return

    if (file.isDirectory)
      for (child <- file.list()) {
        recursiveDelete(new File(file, child))
      }
      file.delete()
