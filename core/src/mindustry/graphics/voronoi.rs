const EPS: f32 = 1.0e-6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: Self) -> f32 {
        self.distance_squared(other).sqrt()
    }

    pub fn distance_squared(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl Default for Point {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl From<(f32, f32)> for Point {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<[f32; 2]> for Point {
    fn from(value: [f32; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipRect {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl ClipRect {
    pub const fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
        .normalized()
    }

    pub const fn from_corners(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self::new(x1, x2, y1, y2)
    }

    pub const fn normalized(self) -> Self {
        Self {
            min_x: if self.min_x <= self.max_x {
                self.min_x
            } else {
                self.max_x
            },
            max_x: if self.min_x >= self.max_x {
                self.min_x
            } else {
                self.max_x
            },
            min_y: if self.min_y <= self.max_y {
                self.min_y
            } else {
                self.max_y
            },
            max_y: if self.min_y >= self.max_y {
                self.min_y
            } else {
                self.max_y
            },
        }
    }

    pub fn width(self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(self) -> f32 {
        self.max_y - self.min_y
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }

    pub fn clamp(self, point: Point) -> Point {
        Point::new(
            point.x.clamp(self.min_x, self.max_x),
            point.y.clamp(self.min_y, self.max_y),
        )
    }

    pub fn clip_segment(self, start: Point, end: Point) -> Option<(Point, Point)> {
        let dx = end.x - start.x;
        let dy = end.y - start.y;

        let mut t0 = 0.0f32;
        let mut t1 = 1.0f32;

        let clip = |p: f32, q: f32, t0: &mut f32, t1: &mut f32| -> bool {
            if p.abs() <= EPS {
                return q >= 0.0;
            }

            let r = q / p;
            if p < 0.0 {
                if r > *t1 {
                    return false;
                }
                if r > *t0 {
                    *t0 = r;
                }
            } else {
                if r < *t0 {
                    return false;
                }
                if r < *t1 {
                    *t1 = r;
                }
            }
            true
        };

        if !clip(-dx, start.x - self.min_x, &mut t0, &mut t1) {
            return None;
        }
        if !clip(dx, self.max_x - start.x, &mut t0, &mut t1) {
            return None;
        }
        if !clip(-dy, start.y - self.min_y, &mut t0, &mut t1) {
            return None;
        }
        if !clip(dy, self.max_y - start.y, &mut t0, &mut t1) {
            return None;
        }

        if t0 > t1 {
            return None;
        }

        let clipped_start = Point::new(start.x + t0 * dx, start.y + t0 * dy);
        let clipped_end = Point::new(start.x + t1 * dx, start.y + t1 * dy);
        Some((clipped_start, clipped_end))
    }
}

impl Default for ClipRect {
    fn default() -> Self {
        Self {
            min_x: 0.0,
            max_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Site {
    pub id: usize,
    pub point: Point,
}

impl Site {
    pub const fn new(id: usize, point: Point) -> Self {
        Self { id, point }
    }

    pub const fn from_xy(id: usize, x: f32, y: f32) -> Self {
        Self::new(id, Point::new(x, y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphEdge {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub site1: usize,
    pub site2: usize,
}

impl GraphEdge {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32, site1: usize, site2: usize) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            site1,
            site2,
        }
    }

    pub fn start(&self) -> Point {
        Point::new(self.x1, self.y1)
    }

    pub fn end(&self) -> Point {
        Point::new(self.x2, self.y2)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub site1: usize,
    pub site2: usize,
    pub start: Option<Point>,
    pub end: Option<Point>,
}

impl Edge {
    pub fn bisector(left: Site, right: Site, min_distance: f32) -> Option<Self> {
        if left.point.distance(right.point) < min_distance {
            return None;
        }

        let dx = right.point.x - left.point.x;
        let dy = right.point.y - left.point.y;
        let adx = dx.abs();
        let ady = dy.abs();

        if adx <= EPS && ady <= EPS {
            return None;
        }

        let mut edge = Self {
            a: 0.0,
            b: 0.0,
            c: left.point.x * dx + left.point.y * dy + (dx * dx + dy * dy) * 0.5,
            site1: left.id,
            site2: right.id,
            start: None,
            end: None,
        };

        if adx > ady {
            if dx.abs() <= EPS {
                return None;
            }
            edge.a = 1.0;
            edge.b = dy / dx;
            edge.c /= dx;
        } else {
            if dy.abs() <= EPS {
                return None;
            }
            edge.b = 1.0;
            edge.a = dx / dy;
            edge.c /= dy;
        }

        Some(edge)
    }

    pub fn with_endpoints(mut self, start: Point, end: Point) -> Self {
        self.start = Some(start);
        self.end = Some(end);
        self
    }

    pub fn clip_to_rect(&self, rect: ClipRect) -> Option<GraphEdge> {
        let rect = rect.normalized();
        let mut points = Vec::with_capacity(4);

        let mut push_unique = |point: Point| {
            if !point.x.is_finite() || !point.y.is_finite() {
                return;
            }

            if point.x < rect.min_x - EPS
                || point.x > rect.max_x + EPS
                || point.y < rect.min_y - EPS
                || point.y > rect.max_y + EPS
            {
                return;
            }

            let duplicate = points.iter().any(|existing: &Point| {
                (existing.x - point.x).abs() <= EPS && (existing.y - point.y).abs() <= EPS
            });
            if !duplicate {
                points.push(point);
            }
        };

        if self.a == 1.0 {
            if self.b.abs() <= EPS {
                if self.c >= rect.min_x - EPS && self.c <= rect.max_x + EPS {
                    push_unique(Point::new(self.c, rect.min_y));
                    push_unique(Point::new(self.c, rect.max_y));
                }
            } else {
                for y in [rect.min_y, rect.max_y] {
                    push_unique(Point::new(self.c - self.b * y, y));
                }
                for x in [rect.min_x, rect.max_x] {
                    push_unique(Point::new(x, (self.c - x) / self.b));
                }
            }
        } else if self.b == 1.0 {
            if self.a.abs() <= EPS {
                if self.c >= rect.min_y - EPS && self.c <= rect.max_y + EPS {
                    push_unique(Point::new(rect.min_x, self.c));
                    push_unique(Point::new(rect.max_x, self.c));
                }
            } else {
                for x in [rect.min_x, rect.max_x] {
                    push_unique(Point::new(x, self.c - self.a * x));
                }
                for y in [rect.min_y, rect.max_y] {
                    push_unique(Point::new((self.c - y) / self.a, y));
                }
            }
        } else {
            return None;
        }

        if points.len() < 2 {
            return None;
        }

        points.sort_by(|lhs, rhs| lhs.x.total_cmp(&rhs.x).then(lhs.y.total_cmp(&rhs.y)));

        Some(GraphEdge::new(
            points[0].x,
            points[0].y,
            points[1].x,
            points[1].y,
            self.site1,
            self.site2,
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratePlan {
    pub sites: Vec<Site>,
    pub clip: ClipRect,
    pub min_distance_between_sites: f32,
}

impl GeneratePlan {
    pub fn new(sites: Vec<Site>, clip: ClipRect) -> Self {
        Self {
            sites,
            clip: clip.normalized(),
            min_distance_between_sites: 1.0,
        }
    }

    pub fn with_min_distance_between_sites(mut self, min_distance_between_sites: f32) -> Self {
        self.min_distance_between_sites = min_distance_between_sites.max(0.0);
        self
    }

    pub fn sort_sites(&mut self) {
        self.sites.sort_by(|lhs, rhs| {
            lhs.point
                .y
                .total_cmp(&rhs.point.y)
                .then(lhs.point.x.total_cmp(&rhs.point.x))
                .then(lhs.id.cmp(&rhs.id))
        });
    }

    pub fn dedupe_sites(&mut self) -> usize {
        if self.sites.is_empty() || self.min_distance_between_sites <= 0.0 {
            return 0;
        }

        let threshold_sq = self.min_distance_between_sites * self.min_distance_between_sites;
        let mut deduped: Vec<Site> = Vec::with_capacity(self.sites.len());

        for site in self.sites.iter().copied() {
            let too_close = deduped
                .iter()
                .any(|existing| existing.point.distance_squared(site.point) < threshold_sq);

            if !too_close {
                deduped.push(site);
            }
        }

        let removed = self.sites.len() - deduped.len();
        self.sites = deduped;
        removed
    }

    pub fn canonicalize(&mut self) {
        self.clip = self.clip.normalized();
        self.sort_sites();
        self.dedupe_sites();
    }

    pub fn site_points(&self) -> Vec<Point> {
        self.sites.iter().map(|site| site.point).collect()
    }

    pub fn build_edge(&self, left: Site, right: Site) -> Option<Edge> {
        Edge::bisector(left, right, self.min_distance_between_sites)
    }
}

impl Default for GeneratePlan {
    fn default() -> Self {
        Self::new(Vec::new(), ClipRect::default())
    }
}

pub type VoronoiPoint = Point;
pub type VoronoiRect = ClipRect;
pub type VoronoiSite = Site;
pub type VoronoiEdge = Edge;
pub type VoronoiGraphEdge = GraphEdge;
pub type VoronoiGeneratePlan = GeneratePlan;
pub type CoreEdge = GraphEdge;

pub fn generate<I, P>(
    values: I,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
) -> VoronoiGeneratePlan
where
    I: IntoIterator<Item = P>,
    P: Into<Point>,
{
    let sites = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| Site::new(index, value.into()))
        .collect::<Vec<_>>();

    let mut plan = GeneratePlan::new(sites, ClipRect::new(min_x, max_x, min_y, max_y));
    plan.canonicalize();
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_rect_normalizes_and_clamps_points() {
        let rect = ClipRect::new(10.0, 0.0, 6.0, -2.0);
        assert_eq!(
            rect,
            ClipRect {
                min_x: 0.0,
                max_x: 10.0,
                min_y: -2.0,
                max_y: 6.0,
            }
        );
        assert_eq!(rect.width(), 10.0);
        assert_eq!(rect.height(), 8.0);
        assert!(rect.contains(Point::new(4.0, 3.0)));
        assert!(!rect.contains(Point::new(11.0, 3.0)));
        assert_eq!(rect.clamp(Point::new(-3.0, 100.0)), Point::new(0.0, 6.0));
    }

    #[test]
    fn generate_sorts_and_dedupes_sites() {
        let plan = generate(
            vec![(5.0, 4.0), (2.0, 1.0), (2.25, 1.0), (9.0, 9.0)],
            20.0,
            0.0,
            10.0,
            -10.0,
        )
        .with_min_distance_between_sites(1.0);

        assert_eq!(
            plan.clip,
            ClipRect {
                min_x: 0.0,
                max_x: 20.0,
                min_y: -10.0,
                max_y: 10.0,
            }
        );
        assert_eq!(plan.sites.len(), 3);
        assert_eq!(plan.sites[0].point, Point::new(2.0, 1.0));
        assert_eq!(plan.sites[1].point, Point::new(5.0, 4.0));
        assert_eq!(plan.sites[2].point, Point::new(9.0, 9.0));
    }

    #[test]
    fn bisector_edge_clips_to_overlay_friendly_segment() {
        let left = Site::from_xy(0, 0.0, 0.0);
        let right = Site::from_xy(1, 10.0, 0.0);
        let edge = Edge::bisector(left, right, 1.0).expect("expected bisector");
        assert_eq!(edge.a, 1.0);
        assert_eq!(edge.b, 0.0);
        assert!((edge.c - 5.0).abs() <= EPS);

        let clipped = edge
            .clip_to_rect(ClipRect::from_corners(0.0, 0.0, 10.0, 10.0))
            .expect("expected clipped segment");
        assert_eq!(clipped.site1, 0);
        assert_eq!(clipped.site2, 1);
        assert_eq!(clipped.x1, 5.0);
        assert_eq!(clipped.y1, 0.0);
        assert_eq!(clipped.x2, 5.0);
        assert_eq!(clipped.y2, 10.0);
    }

    #[test]
    fn generate_plan_keeps_overlay_edge_data_accessible() {
        let plan = generate([(0.0, 0.0), (4.0, 4.0)], -2.0, 6.0, -2.0, 6.0);
        assert_eq!(
            plan.site_points(),
            vec![Point::new(0.0, 0.0), Point::new(4.0, 4.0)]
        );

        let edge = plan
            .build_edge(plan.sites[0], plan.sites[1])
            .expect("expected a valid edge");
        let graph = edge
            .clip_to_rect(plan.clip)
            .expect("expected a clipped graph edge");

        assert_eq!(graph.site1, 0);
        assert_eq!(graph.site2, 1);
        assert!(plan.clip.contains(graph.start()));
        assert!(plan.clip.contains(graph.end()));
    }
}
