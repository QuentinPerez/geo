use num_traits::Float;
use types::{Point, LineString, Polygon, MultiPoint, MultiPolygon};
use algorithm::convexhull::ConvexHull;
use algorithm::orient::{Orient, Direction};
use types::Extremes;

// Useful direction vectors:
// 1., 0. = largest x
// 0., 1. = largest y
// 0., -1. = smallest y
// -1, 0. = smallest x

// various tests for vector orientation relative to a direction vector u
fn up<T>(u: &Point<T>, v: &Point<T>) -> bool
    where T: Float
{
    u.dot(v) > T::zero()
}

fn direction_sign<T>(u: &Point<T>, vi: &Point<T>, vj: &Point<T>) -> T
    where T: Float
{
    u.dot(&(*vi - *vj))
}

// true if Vi is above Vj
fn above<T>(u: &Point<T>, vi: &Point<T>, vj: &Point<T>) -> bool
    where T: Float
{
    direction_sign(u, vi, vj) > T::zero()
}

// true if Vi is below Vj
fn below<T>(u: &Point<T>, vi: &Point<T>, vj: &Point<T>) -> bool
    where T: Float
{
    direction_sign(u, vi, vj) < T::zero()
}

// wrapper for extreme-finding function
fn find_extreme_indices<T, F>(func: F, polygon: &Polygon<T>) -> Extremes
    where T: Float,
          F: Fn(&Point<T>, &Polygon<T>) -> Result<usize, ()>
{
    let directions = vec![Point::new(T::zero(), -T::one()),
                          Point::new(T::one(), T::zero()),
                          Point::new(T::zero(), T::one()),
                          Point::new(-T::one(), T::zero())];
    directions
        .iter()
        .map(|p| func(&p, &polygon).unwrap())
        .collect::<Vec<usize>>()
        .into()
}

// find a convex, counter-clockwise oriented polygon's maximum vertex in a specified direction
// u: a direction vector. We're using a point to represent this, which is a hack but works fine
fn polymax_naive_indices<T>(u: &Point<T>, poly: &Polygon<T>) -> Result<usize, ()>
    where T: Float
{
    let vertices = &poly.exterior.0;
    let mut max: usize = 0;
    for (i, _) in vertices.iter().enumerate() {
        // if vertices[i] is above prior vertices[max]
        if above(u, &vertices[i], &vertices[max]) {
            max = i;
        }
    }
    return Ok(max);
}

pub trait ExtremeIndices<T: Float> {
    /// Find the extreme `x` and `y` indices of a convex Polygon
    ///
    /// The polygon **must be convex and properly (ccw) oriented**.
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
    /// use geo::extremes::ExtremeIndices;
    /// // a diamond shape
    /// let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
    /// let points = points_raw.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>();
    /// let poly = Polygon::new(LineString(points), vec![]);
    /// // Polygon is both convex and oriented counter-clockwise
    /// let extremes = poly.extreme_indices();
    /// assert_eq!(extremes.ymin, 0);
    /// assert_eq!(extremes.xmax, 1);
    /// assert_eq!(extremes.ymax, 2);
    /// assert_eq!(extremes.xmin, 3);
    /// ```
    fn extreme_indices(&self) -> Extremes;
}

impl<T> ExtremeIndices<T> for Polygon<T>
    where T: Float
{
    fn extreme_indices(&self) -> Extremes {
        find_extreme_indices(polymax_naive_indices, self)
    }
}

impl<T> ExtremeIndices<T> for MultiPolygon<T>
    where T: Float
{
    fn extreme_indices(&self) -> Extremes {
        find_extreme_indices(polymax_naive_indices, &self.convex_hull())
    }
}

impl<T> ExtremeIndices<T> for MultiPoint<T>
    where T: Float
{
    fn extreme_indices(&self) -> Extremes {
        find_extreme_indices(polymax_naive_indices, &self.convex_hull())
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use super::*;
    #[test]
    fn test_polygon_extreme_x() {
        // a diamond shape
        let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let min_x = polymax_naive_indices(&Point::new(-1., 0.), &poly1).unwrap();
        let correct = 3_usize;
        assert_eq!(min_x, correct);
    }
    #[test]
    fn test_polygon_extreme_wrapper() {
        // non-convex, with a bump on the top-right edge
        let points_raw = vec![(1.0, 0.0),
                              (1.3, 1.),
                              (2.0, 1.0),
                              (1.75, 1.75),
                              (1.0, 2.0),
                              (0.0, 1.0),
                              (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let extremes = find_extreme_indices(polymax_naive_indices, &poly1.convex_hull());
        let correct = Extremes {
            ymin: 0,
            xmax: 1,
            ymax: 3,
            xmin: 4,
        };
        assert_eq!(extremes, correct);
    }
    #[test]
    fn test_polygon_extreme_wrapper_convex() {
        // convex, with a bump on the top-right edge
        let mut points_raw =
            vec![(1.0, 0.0), (2.0, 1.0), (1.75, 1.75), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let extremes = find_extreme_indices(polymax_naive_indices, &poly1.convex_hull());
        let correct = Extremes {
            ymin: 0,
            xmax: 1,
            ymax: 3,
            xmin: 4,
        };
        assert_eq!(extremes, correct);
    }
}
