//! Entity index change callback mirroring upstream `mindustry.entities.EntityIndexer`.

pub trait EntityIndexer<T> {
    fn change(&mut self, entity: &mut T, index: i32);
}

impl<T, F> EntityIndexer<T> for F
where
    F: FnMut(&mut T, i32),
{
    fn change(&mut self, entity: &mut T, index: i32) {
        self(entity, index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Entity {
        index: i32,
    }

    #[test]
    fn entity_indexer_closure_receives_entity_and_index() {
        let mut entity = Entity { index: -1 };
        let mut calls = Vec::new();
        let mut indexer = |entity: &mut Entity, index| {
            entity.index = index;
            calls.push(index);
        };

        indexer.change(&mut entity, 7);

        assert_eq!(entity.index, 7);
        assert_eq!(calls, vec![7]);
    }
}
