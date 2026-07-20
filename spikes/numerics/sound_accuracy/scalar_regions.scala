import daisy.lang._
import Real._

object TilerSoundAccuracyRegions {
  def affine_mix(x: Real, y: Real, z: Real): Real = {
    require(-2 <= x && x <= 2 && -3 <= y && y <= 3 && -1 <= z && z <= 1)
    x * y + z
  }

  def cancellation(x: Real, y: Real): Real = {
    require(16777216 <= x && x <= 16777216 && 1 <= y && y <= 1)
    (x + y) - x
  }

  def divide_sqrt(x: Real, y: Real): Real = {
    require(1 <= x && x <= 4 && 1 <= y && y <= 2)
    sqrt(x) / y
  }

  def explicit_fma(x: Real, y: Real, z: Real): Real = {
    require(-2 <= x && x <= 2 && -3 <= y && y <= 3 && -1 <= z && z <= 1)
    fma(x, y, z)
  }

  def relational_ratio(x: Real, y: Real): Real = {
    require(1 <= x && x <= 2 && 1 <= y && y <= 2 && x == y)
    x / y
  }

  def materialized_f16(x: Real): Real = {
    require(1 <= x && x <= 1.0009765625)
    val q = x
    q - 1
  }

  def reduce_left(a: Real, b: Real, c: Real, d: Real): Real = {
    require(100000000 <= a && a <= 100000000 &&
      1 <= b && b <= 1 &&
      -100000000 <= c && c <= -100000000 &&
      1 <= d && d <= 1)
    ((a + b) + c) + d
  }

  def reduce_tree(a: Real, b: Real, c: Real, d: Real): Real = {
    require(100000000 <= a && a <= 100000000 &&
      1 <= b && b <= 1 &&
      -100000000 <= c && c <= -100000000 &&
      1 <= d && d <= 1)
    (a + b) + (c + d)
  }
}
