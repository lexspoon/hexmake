scalaVersion := "3.3.3"

logLevel := Level.Info

// Scala Native configuration
enablePlugins(ScalaNativePlugin)

import scala.scalanative.build._

nativeConfig ~= { c =>
  c.withLTO(LTO.none) // thin
    .withMode(Mode.debug) // releaseFast
    .withGC(GC.immix) // commix
}
