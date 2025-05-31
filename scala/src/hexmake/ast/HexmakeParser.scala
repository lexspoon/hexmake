package hexmake.ast

import com.github.plokhotnyuk.jsoniter_scala.core.*
import com.github.plokhotnyuk.jsoniter_scala.macros.*

import java.io.File
import java.nio.file.Files

object HexmakeParser:
  private val stringCodec: JsonValueCodec[String] = JsonCodecMaker.make

  private implicit val hexPathCodec: JsonValueCodec[HexPath] =
    new JsonValueCodec[HexPath]:
      override def decodeValue(in: JsonReader, default: HexPath): HexPath =
        HexPath(stringCodec.decodeValue(in, null))
      override def encodeValue(x: HexPath, out: JsonWriter): Unit =
        out.writeVal(x.path)
      override def nullValue: HexPath = null

  private implicit val codec: JsonValueCodec[HexmakeFile] =
    JsonCodecMaker.make

  def parseFile(file: File): HexmakeFile =
    val fileContents = Files.readString(file.toPath)
    readFromString(fileContents)
