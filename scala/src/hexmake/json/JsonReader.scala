package hexmake.json

import java.io.Reader
import scala.collection.immutable.ArraySeq
import scala.reflect.ClassTag

// TODO(lex) tests. all the public entry points. Each branch.
class JsonReader(input: String):
  /** The position in [[input]] that is currently being looked at */
  private var inputPos = 0

  /** The character in [[input]] that is currently being looked at */
  private var c: Int = if (input.isEmpty) -1 else input.charAt(0)

  /** The current line number */
  private var line = 1

  /** The current column number */
  private var column = 1

  /** The line/column position as an object */
  private def position = Position(line, column)

  /** Advance one character */
  private def advance: Unit =
    if (inputPos == input.length) error("Unexpected EOF")
    inputPos = inputPos + 1
    if (c == '\n')
      line = line + 1
      column = 1
    else column = column + 1
    c = if (input.isEmpty) -1 else input.charAt(inputPos)

  /** Read an expected character. If the character is anything other than the
    * one specified, then throw a parse error.
    */
  private def consume(expected: Int): Unit =
    if (c != expected) error(s"Expected '${expected.asInstanceOf[Char]}'")
    advance

  /** Throw an error at the current position */
  def error(message: String): Nothing =
    throw new JsonParseError(position, message)

  /** Read one JSON value, for any type of value */
  def value: Any =
    c match {
      case '{' => readObjectAsMap
      case '[' => array
      case '"' => string
      case '-' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' =>
//        number
      case 't' => boolean
      case 'f' => boolean
      case 'n' => readNull
    }

  /** Read a boolean value, either "true" or "false". */
  def boolean: Boolean =
    c match {
      case 't' =>
        advance
        consume('r')
        consume('u')
        consume('e')
        true
      case 'f' =>
        advance
        consume('a')
        consume('l')
        consume('s')
        consume('e')
        false
      case _ =>
        error("Expected a boolean")
    }

  def readNull: Null =
    consume('n')
    consume('u')
    consume('l')
    consume('l')
    null

  /** Read an object and return a map of all the field values */
  def readObjectAsMap: Map[String, Any] =
    var fieldMap: Map[String, Any] = Map.empty
    readObject(
      (fieldName: String) => fieldMap = fieldMap + (fieldName -> value),
      (position) => fieldMap
    )

  /** Read an object. This version The [readField] parameter will be called for
    * each field that the object contains; it should read exactly one JSON value
    * and then store the result locally. The [buildObject] parameter will be
    * called once, to
    */
  def readObject[T](readField: String => Unit, buildObject: Position => T): T =
    val startPosition = position
    consume('{')
    ws
    if (c != '}') members(readField)
    buildObject(startPosition)

  private def members(readField: String => Unit): Unit =
    member(readField)
    while c == ',' do
      advance
      member(readField)

  private def member(readField: String => Unit): Unit =
    ws
    val fieldName = string
    ws
    consume(':')
    readField(fieldName)

  def array[T: ClassTag](readElement: () => T): ArraySeq[T] =
    val result = ArraySeq.newBuilder
    consume('[')
    ws
    while c != ']' do
      consume(',')
      ws
      result.addOne(readElement())
      ws
    consume(']')
    result.result()

  def string: String =
    val result = new StringBuilder
    consume('"')
    while c != '"' do result.append(character)
    advance
    result.result

  /** Read one character from a string literal */
  private def character: Char =
    if c < 0 then error("Unexpected EOF inside of string literal")
    if c == '"' then
      error("Cannot include an unescaped quote inside a string literal")
    if c != '\\' then
      val result = c.toChar
      advance
      return result

    // At this point, the character must be \. Process the escape.
    advance

    if c < 0 then error("Unexpected EOF")
    if c == 'u' then
      advance
      val hexDigits: String = String(Array(hex, hex, hex, hex))
      val utf16Code = Integer.parseInt(hexDigits, 16)
      return utf16Code.toChar

    // It's a one-character escape
    val result: Char = c match {
      case '"'  => '"'
      case '\\' => '\\'
      case '/'  => '/'
      case 'b'  => '\b'
      case 'f'  => '\f'
      case 'n'  => '\n'
      case 'r'  => '\r'
      case 't'  => '\t'
      case _    => error("Unknown escape")
    }
    advance
    result

  def hex: Char =
    if !(('0' to '9').contains(c) ||
        ('A' to 'F').contains(c) ||
        ('a' to 'f').contains(c))
    then error("Unexpected hex code in Unicode escape")
    val result = c.toChar
    advance
    result
    /*
  number
  integer fraction exponent

  integer
  digit
  onenine digits
    '-' digit
    '-' onenine digits

  digits
  digit
  digit digits

  digit
  '0'
  onenine

  onenine
  '1' . '9'

  fraction
  ""
  '.' digits

  exponent
  ""
  'E' sign digits
  'e' sign digits

  sign
  ""
  '+'
  '-'
     */
  private def ws: Unit =
    while (c == 0x20 || c == 0x0a || c == 0x0d || c == 0x09)
      advance

/** This exception indicates that a string was not formatted correctly.
  */
class JsonParseError(val position: Position, val message: String)
    extends RuntimeException(message)
