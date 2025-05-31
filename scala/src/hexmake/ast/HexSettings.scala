package hexmake.ast

case class HexSettings(
    /** The number of concurrent executions to run */
    concurrency: Int = 8
)
