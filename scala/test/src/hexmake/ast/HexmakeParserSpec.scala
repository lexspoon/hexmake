package hexmake.ast

import hexmake.util.HexPaths.*
import hexmake.util.TempFiles.*
import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers

import java.nio.file.Files
import scala.collection.immutable.ArraySeq

class HexmakeParserSpec extends AnyFunSuite with Matchers:
  test("should parse a correct file") {
    val hexmakeFile: HexmakeFile = parse("""{
              |  "rules": [
              |    {
              |      "outputs": [
              |        "out/lib.o"
              |      ],
              |      "inputs": [
              |        "lib.c",
              |        "lib.h"
              |      ],
              |      "commands": [
              |        "gcc -o out/lib.o -c lib.c"
              |      ]
              |    },
              |    {
              |      "outputs": [
              |        "out/main.o"
              |      ],
              |      "inputs": [
              |        "lib.h",
              |        "main.c"
              |      ],
              |      "commands": [
              |        "gcc -o out/main.o -c main.c"
              |      ]
              |    },
              |    {
              |      "outputs": [
              |        "out/main"
              |      ],
              |      "inputs": [
              |        "out/lib.o",
              |        "out/main.o"
              |      ],
              |      "commands": [
              |        "gcc -o out/main out/lib.o out/main.o"
              |      ]
              |    }
              |  ]
              |}
              |""".stripMargin)

    hexmakeFile shouldBe HexmakeFile(
      environ = ArraySeq(),
      rules = ArraySeq(
        HexRule(
          hexPaths("out/lib.o"),
          hexPaths("lib.c", "lib.h"),
          ArraySeq("gcc -o out/lib.o -c lib.c")),
        HexRule(
          hexPaths("out/main.o"),
          hexPaths("lib.h", "main.c"),
          ArraySeq("gcc -o out/main.o -c main.c")),
        HexRule(
          hexPaths("out/main"),
          hexPaths("out/lib.o", "out/main.o"),
          ArraySeq("gcc -o out/main out/lib.o out/main.o"))
      )
    )
  }

  test("should reject a malformed file") {
    val thrown = the[RuntimeException] thrownBy parse("""{
          |  "rules": [
          |}
          |""".stripMargin)

    thrown.getMessage should include("expected '{'")
  }

  def parse(source: String): HexmakeFile =
    withTempFile(source) { tempFile =>
      Files.write(tempFile.toPath, source.getBytes)
      HexmakeParser.parseFile(tempFile)
    }
