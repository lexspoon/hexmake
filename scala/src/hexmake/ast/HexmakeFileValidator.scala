package hexmake.ast

import scala.collection.immutable.ArraySeq

/** Check a [[HexmakeFile]] for internal consistency.
  */
class HexmakeFileValidator(hexmakeFile: HexmakeFile):
  private val problems = ArraySeq.newBuilder[String]

  /** Check the entire file and return the list of problems detected. If the
    * list is empty, then the file is good.
    */
  def check: ArraySeq[String] =
    checkFile()
    problems.result

  private def checkFile(): Unit =
    ???

  private def checkRule(rule: HexRule): Unit =
    ???
