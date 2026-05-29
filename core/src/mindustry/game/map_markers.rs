//! Runtime container mirroring upstream `mindustry.game.MapMarkers`.
//!
//! The Java implementation keeps two views of the same objective markers:
//! an ID map used by generated network calls and a compact sequential list
//! used for fast iteration/rendering. This Rust port preserves the same
//! replacement and removal index semantics while leaving JSON byte
//! persistence to the save/serialization layer.

use crate::mindustry::game::ObjectiveMarker;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MapMarkers {
    map: BTreeMap<i32, ObjectiveMarker>,
    order: Vec<i32>,
}

impl MapMarkers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, id: i32, mut marker: ObjectiveMarker) {
        if let Some(previous) = self.map.get(&id) {
            marker.common_mut().array_index = previous.common().array_index;
            self.map.insert(id, marker);
        } else {
            marker.common_mut().array_index = self.order.len() as i32;
            self.order.push(id);
            self.map.insert(id, marker);
        }
    }

    pub fn remove(&mut self, id: i32) -> Option<ObjectiveMarker> {
        let removed = self.map.remove(&id)?;
        let array_index = removed.common().array_index;
        if array_index >= 0 {
            let array_index = array_index as usize;
            if array_index < self.order.len() {
                self.order.swap_remove(array_index);
                if let Some(replaced_id) = self.order.get(array_index).copied() {
                    if let Some(replaced) = self.map.get_mut(&replaced_id) {
                        replaced.common_mut().array_index = array_index as i32;
                    }
                }
            }
        }
        Some(removed)
    }

    pub fn get(&self, id: i32) -> Option<&ObjectiveMarker> {
        self.map.get(&id)
    }

    pub fn get_mut(&mut self, id: i32) -> Option<&mut ObjectiveMarker> {
        self.map.get_mut(&id)
    }

    pub fn has(&self, id: i32) -> bool {
        self.map.contains_key(&id)
    }

    pub fn size(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }

    pub fn ids(&self) -> impl Iterator<Item = i32> + '_ {
        self.order.iter().copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ObjectiveMarker> + '_ {
        self.order.iter().filter_map(|id| self.map.get(id))
    }

    pub fn for_each_mut(&mut self, mut f: impl FnMut(&mut ObjectiveMarker)) {
        let ids = self.order.clone();
        for id in ids {
            if let Some(marker) = self.map.get_mut(&id) {
                f(marker);
            }
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (i32, &ObjectiveMarker)> + '_ {
        self.order
            .iter()
            .filter_map(|&id| self.map.get(&id).map(|marker| (id, marker)))
    }

    pub fn rebuild_from_entries(entries: impl IntoIterator<Item = (i32, ObjectiveMarker)>) -> Self {
        let mut markers = Self::new();
        for (id, marker) in entries {
            markers.add(id, marker);
        }
        markers
    }
}

impl<'a> IntoIterator for &'a MapMarkers {
    type Item = &'a ObjectiveMarker;
    type IntoIter = Box<dyn Iterator<Item = &'a ObjectiveMarker> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::game::map_objectives::{PointMarker, ShapeMarker, TextMarker};
    use crate::mindustry::game::Vec2;
    use crate::mindustry::logic::LMarkerControl;

    #[test]
    fn add_replace_and_iterate_preserve_java_array_indices() {
        let mut markers = MapMarkers::new();
        markers.add(7, ObjectiveMarker::Point(PointMarker::default()));
        markers.add(9, ObjectiveMarker::Shape(ShapeMarker::default()));

        assert_eq!(markers.size(), 2);
        assert_eq!(markers.ids().collect::<Vec<_>>(), vec![7, 9]);
        assert_eq!(markers.get(7).unwrap().common().array_index, 0);
        assert_eq!(markers.get(9).unwrap().common().array_index, 1);

        markers.add(7, ObjectiveMarker::Text(TextMarker::default()));
        assert_eq!(markers.size(), 2);
        assert_eq!(markers.ids().collect::<Vec<_>>(), vec![7, 9]);
        assert_eq!(markers.get(7).unwrap().type_name(), "Text");
        assert_eq!(markers.get(7).unwrap().common().array_index, 0);
        assert_eq!(markers.get(9).unwrap().common().array_index, 1);

        let names = markers
            .iter()
            .map(ObjectiveMarker::type_name)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Text", "Shape"]);
    }

    #[test]
    fn remove_swap_replaces_with_last_marker_like_java_unordered_seq_remove() {
        let mut markers = MapMarkers::new();
        markers.add(1, ObjectiveMarker::Point(PointMarker::default()));
        markers.add(2, ObjectiveMarker::Shape(ShapeMarker::default()));
        markers.add(3, ObjectiveMarker::Text(TextMarker::default()));
        markers.add(4, ObjectiveMarker::Point(PointMarker::default()));

        let removed = markers.remove(2).unwrap();
        assert_eq!(removed.type_name(), "Shape");
        assert!(!markers.has(2));
        assert_eq!(markers.ids().collect::<Vec<_>>(), vec![1, 4, 3]);
        assert_eq!(markers.get(1).unwrap().common().array_index, 0);
        assert_eq!(markers.get(4).unwrap().common().array_index, 1);
        assert_eq!(markers.get(3).unwrap().common().array_index, 2);

        markers.remove(3);
        assert_eq!(markers.ids().collect::<Vec<_>>(), vec![1, 4]);
        assert_eq!(markers.get(1).unwrap().common().array_index, 0);
        assert_eq!(markers.get(4).unwrap().common().array_index, 1);
        assert!(markers.remove(99).is_none());
    }

    #[test]
    fn mutable_iteration_operates_on_sequential_view() {
        let mut markers = MapMarkers::new();
        markers.add(4, ObjectiveMarker::Point(PointMarker::default()));
        markers.add(5, ObjectiveMarker::Point(PointMarker::default()));

        markers.for_each_mut(|marker| {
            marker.control(LMarkerControl::Pos, 2.0, 3.0, f64::NAN);
        });

        for marker in &markers {
            match marker {
                ObjectiveMarker::Point(point) => assert_eq!(point.pos, Vec2::new(16.0, 24.0)),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn rebuild_from_entries_roundtrips_compact_indices_after_removal() {
        let mut markers = MapMarkers::new();
        markers.add(1, ObjectiveMarker::Point(PointMarker::default()));
        markers.add(2, ObjectiveMarker::Shape(ShapeMarker::default()));
        markers.add(3, ObjectiveMarker::Text(TextMarker::default()));

        markers.remove(2);
        markers.add(4, ObjectiveMarker::Point(PointMarker::default()));

        let rebuilt = MapMarkers::rebuild_from_entries(
            markers.entries().map(|(id, marker)| (id, marker.clone())),
        );

        assert_eq!(rebuilt, markers);
        assert_eq!(rebuilt.ids().collect::<Vec<_>>(), vec![1, 3, 4]);
        assert_eq!(rebuilt.get(1).unwrap().common().array_index, 0);
        assert_eq!(rebuilt.get(3).unwrap().common().array_index, 1);
        assert_eq!(rebuilt.get(4).unwrap().common().array_index, 2);
    }
}
