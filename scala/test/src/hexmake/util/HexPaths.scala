package hexmake.util

import hexmake.ast.HexPath

import scala.collection.immutable.ArraySeq

object HexPaths:
  def hexPaths(paths: String*): ArraySeq[HexPath] =
    ArraySeq.from(paths).map(p => HexPath(p))
