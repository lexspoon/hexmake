package hexmake.ast

import hexmake.json.Position
import scala.collection.immutable.ArraySeq

/** One element of a Hexmake file */
sealed abstract class HexmakeElement {
  var position: Position = _
}

/** An entire Hexmake file */
case class HexmakeFile(environ: ArraySeq[String], rules: ArraySeq[HexRule])
    extends HexmakeElement

/** One rule */
case class HexRule(
    outputs: ArraySeq[HexPath],
    inputs: ArraySeq[HexPath],
    commands: ArraySeq[String])
    extends HexmakeElement

