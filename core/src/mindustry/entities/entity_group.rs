use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI32, Ordering};

static LAST_ID: AtomicI32 = AtomicI32::new(0);

pub trait EntityGroupItem {
    fn id(&self) -> i32;
    fn remove(&mut self);
}

pub trait SpatialEntity {
    fn bounds(&self) -> Rect;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn overlaps(self, other: Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityGroup<T> {
    array: Vec<T>,
    map: Option<HashMap<i32, usize>>,
    clearing: bool,
}

impl<T> EntityGroup<T>
where
    T: EntityGroupItem + PartialEq,
{
    pub fn new(mapping: bool) -> Self {
        Self {
            array: Vec::with_capacity(32),
            map: mapping.then(HashMap::new),
            clearing: false,
        }
    }

    pub fn next_id() -> i32 {
        loop {
            let current = LAST_ID.load(Ordering::Relaxed);
            let next = if current >= i32::MAX - 2 {
                1
            } else {
                current + 1
            };
            if LAST_ID
                .compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return if current >= i32::MAX - 2 { 0 } else { current };
            }
        }
    }

    pub fn check_next_id(id: i32) {
        let target = id.saturating_add(1);
        let mut current = LAST_ID.load(Ordering::Relaxed);
        while current < target {
            match LAST_ID.compare_exchange(current, target, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(next) => current = next,
            }
        }
    }

    #[cfg(test)]
    fn reset_next_id_for_tests(value: i32) {
        LAST_ID.store(value, Ordering::SeqCst);
    }

    pub fn use_tree(&self) -> bool {
        false
    }

    pub fn mapping_enabled(&self) -> bool {
        self.map.is_some()
    }

    pub fn size(&self) -> usize {
        self.array.len()
    }

    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    pub fn index(&self, index: usize) -> Option<&T> {
        self.array.get(index)
    }

    pub fn first(&self) -> Option<&T> {
        self.array.first()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.array.iter()
    }

    pub fn copy(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.array.clone()
    }

    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        self.array.sort_by(&mut compare);
        self.rebuild_map();
    }

    pub fn contains<F>(&self, pred: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        self.array.iter().any(pred)
    }

    pub fn count<F>(&self, mut pred: F) -> usize
    where
        F: FnMut(&T) -> bool,
    {
        self.array.iter().filter(|item| pred(*item)).count()
    }

    pub fn find<F>(&self, mut pred: F) -> Option<&T>
    where
        F: FnMut(&T) -> bool,
    {
        self.array.iter().find(|item| pred(*item))
    }

    pub fn get_by_id(&self, id: i32) -> Option<&T> {
        let map = self
            .map
            .as_ref()
            .expect("Mapping is not enabled for this entity group");
        map.get(&id).and_then(|index| self.array.get(*index))
    }

    pub fn add(&mut self, item: T) {
        let index = self.array.len();
        if let Some(map) = &mut self.map {
            map.insert(item.id(), index);
        }
        self.array.push(item);
    }

    pub fn add_index(&mut self, item: T) -> usize {
        let index = self.array.len();
        self.add(item);
        index
    }

    pub fn remove(&mut self, item: &T) -> Option<T> {
        if self.clearing {
            return None;
        }
        let index = self.array.iter().position(|candidate| candidate == item)?;
        self.remove_at(index)
    }

    pub fn remove_by_id(&mut self, id: i32) -> Option<T> {
        let index = *self
            .map
            .as_ref()
            .expect("Mapping is not enabled for this entity group")
            .get(&id)?;
        let mut removed = self.remove_at(index)?;
        removed.remove();
        Some(removed)
    }

    pub fn remove_index(&mut self, item: &T, position: usize) -> Option<T> {
        if self.clearing || position >= self.array.len() {
            return None;
        }

        if &self.array[position] != item {
            return self.remove(item);
        }

        self.remove_at(position)
    }

    fn remove_at(&mut self, index: usize) -> Option<T> {
        if index >= self.array.len() {
            return None;
        }

        let removed = self.array.swap_remove(index);
        if let Some(map) = &mut self.map {
            map.remove(&removed.id());
            if index < self.array.len() {
                map.insert(self.array[index].id(), index);
            }
        }
        Some(removed)
    }

    pub fn clear(&mut self) {
        self.clearing = true;
        for item in &mut self.array {
            item.remove();
        }
        self.array.clear();
        if let Some(map) = &mut self.map {
            map.clear();
        }
        self.clearing = false;
    }

    pub fn check_id_collisions(&self) -> Vec<&T> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for item in &self.array {
            if !seen.insert(item.id()) {
                out.push(item);
            }
        }
        out
    }

    fn rebuild_map(&mut self) {
        if let Some(map) = &mut self.map {
            map.clear();
            for (index, item) in self.array.iter().enumerate() {
                map.insert(item.id(), index);
            }
        }
    }
}

impl<T> EntityGroup<T>
where
    T: EntityGroupItem + PartialEq + SpatialEntity,
{
    pub fn intersect(&self, rect: Rect) -> Vec<&T> {
        self.array
            .iter()
            .filter(|item| item.bounds().overlaps(rect))
            .collect()
    }

    pub fn intersects_any<F>(&self, rect: Rect, mut pred: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        self.array
            .iter()
            .any(|item| item.bounds().overlaps(rect) && pred(item))
    }
}

impl<T> IntoIterator for EntityGroup<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestEntity {
        id: i32,
        removed: bool,
        bounds: Rect,
    }

    impl TestEntity {
        fn new(id: i32, x: f32, y: f32) -> Self {
            Self {
                id,
                removed: false,
                bounds: Rect::new(x, y, 4.0, 4.0),
            }
        }
    }

    impl EntityGroupItem for TestEntity {
        fn id(&self) -> i32 {
            self.id
        }

        fn remove(&mut self) {
            self.removed = true;
        }
    }

    impl SpatialEntity for TestEntity {
        fn bounds(&self) -> Rect {
            self.bounds
        }
    }

    #[test]
    fn global_ids_increment_wrap_and_check_next_id() {
        EntityGroup::<TestEntity>::reset_next_id_for_tests(0);
        assert_eq!(EntityGroup::<TestEntity>::next_id(), 0);
        assert_eq!(EntityGroup::<TestEntity>::next_id(), 1);

        EntityGroup::<TestEntity>::check_next_id(41);
        assert_eq!(EntityGroup::<TestEntity>::next_id(), 42);

        EntityGroup::<TestEntity>::reset_next_id_for_tests(i32::MAX - 2);
        assert_eq!(EntityGroup::<TestEntity>::next_id(), 0);
        assert_eq!(EntityGroup::<TestEntity>::next_id(), 1);
    }

    #[test]
    fn add_mapping_lookup_and_duplicate_id_detection_follow_java_shape() {
        let mut group = EntityGroup::new(true);
        assert!(group.mapping_enabled());
        assert_eq!(group.add_index(TestEntity::new(7, 0.0, 0.0)), 0);
        group.add(TestEntity::new(8, 10.0, 0.0));
        group.add(TestEntity::new(7, 20.0, 0.0));

        assert_eq!(group.size(), 3);
        assert_eq!(group.get_by_id(8).unwrap().id, 8);
        assert_eq!(group.check_id_collisions().len(), 1);
        assert_eq!(group.count(|entity| entity.id == 7), 2);
        assert!(group.contains(|entity| entity.id == 8));
    }

    #[test]
    fn remove_and_remove_index_use_swap_remove_and_repair_mapping() {
        let mut group = EntityGroup::new(true);
        let a = TestEntity::new(1, 0.0, 0.0);
        let b = TestEntity::new(2, 10.0, 0.0);
        let c = TestEntity::new(3, 20.0, 0.0);
        group.add(a.clone());
        group.add(b.clone());
        group.add(c.clone());

        assert_eq!(group.remove(&b).unwrap().id, 2);
        assert_eq!(group.size(), 2);
        assert!(group.get_by_id(2).is_none());
        assert_eq!(group.get_by_id(3).unwrap().id, 3);
        assert_eq!(group.index(1).unwrap().id, 3);

        assert_eq!(group.remove_index(&c, 1).unwrap().id, 3);
        assert_eq!(group.first().unwrap().id, 1);
    }

    #[test]
    fn remove_by_id_marks_removed_and_clear_marks_all_removed() {
        let mut group = EntityGroup::new(true);
        group.add(TestEntity::new(1, 0.0, 0.0));
        group.add(TestEntity::new(2, 10.0, 0.0));

        let removed = group.remove_by_id(1).unwrap();
        assert!(removed.removed);
        assert_eq!(group.size(), 1);

        group.clear();
        assert!(group.is_empty());
        assert!(group.get_by_id(2).is_none());
    }

    #[test]
    fn sort_copy_find_and_intersections_are_deterministic() {
        let mut group = EntityGroup::new(true);
        group.add(TestEntity::new(3, 20.0, 0.0));
        group.add(TestEntity::new(1, 0.0, 0.0));
        group.add(TestEntity::new(2, 10.0, 0.0));
        group.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(
            group
                .copy()
                .into_iter()
                .map(|entity| entity.id)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(group.find(|entity| entity.id == 2).unwrap().bounds.x, 10.0);
        assert_eq!(group.get_by_id(3).unwrap().bounds.x, 20.0);

        let hits = group.intersect(Rect::new(9.0, -1.0, 3.0, 3.0));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, 2);
        assert!(group.intersects_any(Rect::new(19.0, -1.0, 3.0, 3.0), |entity| entity.id == 3));
    }
}
