package hexmake.ast

import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.{MatchResult, Matcher}
import org.scalatest.matchers.should.Matchers

class HexPathSpec extends AnyFunSuite with Matchers:
  test("identify output paths") {
    HexPath("out/foo.o") should beOutput
    HexPath("foo.c") shouldNot beOutput
    HexPath("src/foo.c") shouldNot beOutput
    HexPath("output.c") shouldNot beOutput
  }

  object beOutput extends Matcher[HexPath]:
    override def apply(left: HexPath): MatchResult =
      MatchResult(
        left.isOutput,
        s"$left was not an output",
        s"$left was an output"
      )
