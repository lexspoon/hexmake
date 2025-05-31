package hexmake.ast

/** A path that can be built and/or used as source code.
  */
case class HexPath(path: String):
  def isOutput: Boolean = path.startsWith(HexPath.OUTPUT_ROOT + "/")
  override def toString = path
  def child(childPath: String): HexPath = HexPath(s"$path/$childPath")

object HexPath:
  def OUTPUT_ROOT = "out"
