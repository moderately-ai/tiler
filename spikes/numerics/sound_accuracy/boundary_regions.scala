import daisy.lang._
import Real._

object TilerSoundAccuracyBoundaries {
  def gradual_subnormal_add(x: Real, y: Real): Real = {
    require(1e-40 <= x && x <= 1e-40 && 1e-40 <= y && y <= 1e-40)
    x + y
  }

  def possible_overflow(x: Real): Real = {
    require(3e38 <= x && x <= 3.4e38)
    x * x
  }
}
