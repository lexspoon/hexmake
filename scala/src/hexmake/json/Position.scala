package hexmake.json


/** A position in a source file */
case class Position(
                     /** The line number, with 1 as the first line.
                       */
                     line: Int,

                     /** The column number, with 1 as the first column */
                     column: Int)
