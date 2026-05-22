//! Mirrors the non-UI helpers from upstream `mindustry.logic.LCanvas`.

use std::collections::{BTreeMap, BTreeSet};

use super::LOGIC_CANVAS_INVALID_JUMP;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicAlign {
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl LogicAlign {
    pub const ALL: [LogicAlign; 9] = [
        LogicAlign::TopLeft,
        LogicAlign::Top,
        LogicAlign::TopRight,
        LogicAlign::Left,
        LogicAlign::Center,
        LogicAlign::Right,
        LogicAlign::BottomLeft,
        LogicAlign::Bottom,
        LogicAlign::BottomRight,
    ];

    pub const fn java_bits(self) -> i32 {
        match self {
            LogicAlign::Center => 1,
            LogicAlign::Top => 2,
            LogicAlign::Bottom => 4,
            LogicAlign::Left => 8,
            LogicAlign::Right => 16,
            LogicAlign::TopLeft => 10,
            LogicAlign::TopRight => 18,
            LogicAlign::BottomLeft => 12,
            LogicAlign::BottomRight => 20,
        }
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            LogicAlign::Center => "center",
            LogicAlign::Top => "top",
            LogicAlign::Bottom => "bottom",
            LogicAlign::Left => "left",
            LogicAlign::Right => "right",
            LogicAlign::TopLeft => "topLeft",
            LogicAlign::TopRight => "topRight",
            LogicAlign::BottomLeft => "bottomLeft",
            LogicAlign::BottomRight => "bottomRight",
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "center" => Some(LogicAlign::Center),
            "top" => Some(LogicAlign::Top),
            "bottom" => Some(LogicAlign::Bottom),
            "left" => Some(LogicAlign::Left),
            "right" => Some(LogicAlign::Right),
            "topLeft" => Some(LogicAlign::TopLeft),
            "topRight" => Some(LogicAlign::TopRight),
            "bottomLeft" => Some(LogicAlign::BottomLeft),
            "bottomRight" => Some(LogicAlign::BottomRight),
            _ => None,
        }
    }

    pub fn by_java_bits(bits: i32) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|align| align.java_bits() == bits)
    }

    pub const fn is_center_horizontal(self) -> bool {
        self.java_bits() & 8 == 0 && self.java_bits() & 16 == 0
    }

    pub const fn is_center_vertical(self) -> bool {
        self.java_bits() & 2 == 0 && self.java_bits() & 4 == 0
    }
}

pub fn logic_canvas_use_rows(viewport_width: f32, ui_scale: f32) -> bool {
    viewport_width < ui_scale * 900.0 * 1.2
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicJumpRange {
    pub begin: i32,
    pub end: i32,
    pub flipped: bool,
}

impl LogicJumpRange {
    pub const fn invalid() -> Self {
        Self {
            begin: LOGIC_CANVAS_INVALID_JUMP,
            end: LOGIC_CANVAS_INVALID_JUMP,
            flipped: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicJumpPlacement {
    pub begin: i32,
    pub end: i32,
    pub flipped: bool,
    pub pred_height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LogicJumpRepresentative {
    input_index: usize,
    range: LogicJumpRange,
    pred_height: usize,
}

pub fn normalize_logic_jump_range(from: i32, to: Option<i32>) -> Option<LogicJumpRange> {
    to.map(|to| LogicJumpRange {
        begin: from.min(to),
        end: from.max(to),
        flipped: from >= to,
    })
}

pub fn representative_logic_jumps(ranges: &[Option<LogicJumpRange>]) -> Vec<usize> {
    let mut repr_before: BTreeMap<i32, usize> = BTreeMap::new();
    let mut repr_after: BTreeMap<i32, usize> = BTreeMap::new();

    for (index, range) in ranges.iter().enumerate() {
        let Some(range) = range else {
            continue;
        };
        if range.begin == LOGIC_CANVAS_INVALID_JUMP {
            continue;
        }

        if range.flipped {
            if let Some(prev) = repr_after.get(&range.begin).and_then(|idx| ranges[*idx]) {
                if prev.end >= range.end {
                    continue;
                }
            }
            repr_after.insert(range.begin, index);
        } else {
            if let Some(prev) = repr_before.get(&range.end).and_then(|idx| ranges[*idx]) {
                if prev.begin <= range.begin {
                    continue;
                }
            }
            repr_before.insert(range.end, index);
        }
    }

    let mut reps: Vec<_> = repr_before
        .values()
        .chain(repr_after.values())
        .copied()
        .collect();
    reps.sort_by_key(|index| ranges[*index].unwrap().begin);
    reps
}

pub fn assign_logic_jump_heights(
    ranges: &[Option<LogicJumpRange>],
) -> Vec<Option<LogicJumpPlacement>> {
    let representatives = representative_logic_jumps(ranges);
    let mut repr_before: BTreeMap<i32, usize> = BTreeMap::new();
    let mut repr_after: BTreeMap<i32, usize> = BTreeMap::new();
    let mut processed = Vec::with_capacity(representatives.len());

    for input_index in representatives {
        let range = ranges[input_index].unwrap();
        let rep_index = processed.len();
        if range.flipped {
            repr_after.insert(range.begin, rep_index);
        } else {
            repr_before.insert(range.end, rep_index);
        }
        processed.push(LogicJumpRepresentative {
            input_index,
            range,
            pred_height: 0,
        });
    }

    let mut marked_done = vec![false; processed.len()];
    let mut occupiers: Vec<usize> = Vec::new();
    let mut occupied: BTreeSet<usize> = BTreeSet::new();

    for index in 0..processed.len() {
        let begin = processed[index].range.begin;
        occupiers.retain(|occupier| {
            if processed[*occupier].range.end > begin {
                true
            } else {
                occupied.remove(&processed[*occupier].pred_height);
                false
            }
        });
        let height = logic_jump_height(
            index,
            &mut processed,
            &mut marked_done,
            &occupiers,
            &occupied,
        );
        occupiers.push(index);
        occupied.insert(height);
    }

    let mut output = vec![None; ranges.len()];
    for (index, range) in ranges.iter().enumerate() {
        let Some(range) = range else {
            continue;
        };
        if range.begin == LOGIC_CANVAS_INVALID_JUMP {
            continue;
        }

        let rep_index = if range.flipped {
            repr_after.get(&range.begin)
        } else {
            repr_before.get(&range.end)
        };
        if let Some(rep_index) = rep_index {
            let rep = processed[*rep_index];
            output[index] = Some(LogicJumpPlacement {
                begin: range.begin,
                end: range.end,
                flipped: range.flipped,
                pred_height: rep.pred_height,
            });
        }
    }

    // Keep representative source indices observable by ensuring every representative
    // produced a placement at its original index. This mirrors the Java pass that
    // recalculates representative `JumpCurve`s first, then copies their height back
    // to every duplicate curve.
    debug_assert!(processed
        .iter()
        .all(|rep| output[rep.input_index].is_some()));

    output
}

fn logic_jump_height(
    index: usize,
    processed: &mut [LogicJumpRepresentative],
    marked_done: &mut [bool],
    occupiers: &[usize],
    occupied: &BTreeSet<usize>,
) -> usize {
    if marked_done[index] {
        return processed[index].pred_height;
    }

    let jmp_end = processed[index].range.end;
    let mut tmp_occupiers = occupiers.to_vec();
    let mut tmp_occupied = occupied.clone();

    let mut max_nested: Option<usize> = None;
    for next in index + 1..processed.len() {
        let cur = processed[next].range;
        if cur.end > jmp_end {
            continue;
        }

        tmp_occupiers.retain(|occupier| {
            if processed[*occupier].range.end > cur.begin {
                true
            } else {
                tmp_occupied.remove(&processed[*occupier].pred_height);
                false
            }
        });

        let height = logic_jump_height(next, processed, marked_done, &tmp_occupiers, &tmp_occupied);
        tmp_occupiers.push(next);
        tmp_occupied.insert(height);
        max_nested = Some(max_nested.map_or(height, |max| max.max(height)));
    }

    let mut height = max_nested.map_or(0, |max| max + 1);
    while occupied.contains(&height) {
        height += 1;
    }

    processed[index].pred_height = height;
    marked_done[index] = true;
    height
}

#[cfg(test)]
mod tests {
    use super::{
        assign_logic_jump_heights, logic_canvas_use_rows, normalize_logic_jump_range,
        representative_logic_jumps, LogicAlign, LogicJumpPlacement, LogicJumpRange,
    };
    use crate::mindustry::logic::LOGIC_CANVAS_INVALID_JUMP;

    #[test]
    fn logic_align_bits_names_and_center_flags_match_java_alignment_values() {
        assert_eq!(LogicAlign::Center.java_bits(), 1);
        assert_eq!(LogicAlign::Top.java_bits(), 2);
        assert_eq!(LogicAlign::Bottom.java_bits(), 4);
        assert_eq!(LogicAlign::Left.java_bits(), 8);
        assert_eq!(LogicAlign::Right.java_bits(), 16);
        assert_eq!(LogicAlign::TopLeft.java_bits(), 10);
        assert_eq!(LogicAlign::BottomRight.java_bits(), 20);
        assert_eq!(LogicAlign::TopRight.wire_name(), "topRight");
        assert_eq!(LogicAlign::by_name("topRight"), Some(LogicAlign::TopRight));
        assert_eq!(LogicAlign::by_java_bits(12), Some(LogicAlign::BottomLeft));
        assert!(LogicAlign::Top.is_center_horizontal());
        assert!(!LogicAlign::TopLeft.is_center_horizontal());
        assert!(LogicAlign::Left.is_center_vertical());
        assert!(!LogicAlign::BottomLeft.is_center_vertical());
    }

    #[test]
    fn logic_canvas_rows_and_jump_normalization_match_lcanvas_helpers() {
        assert!(logic_canvas_use_rows(1079.9, 1.0));
        assert!(!logic_canvas_use_rows(1080.0, 1.0));
        assert!(logic_canvas_use_rows(2159.0, 2.0));
        assert!(!logic_canvas_use_rows(2160.0, 2.0));

        assert_eq!(
            normalize_logic_jump_range(3, Some(8)),
            Some(LogicJumpRange {
                begin: 3,
                end: 8,
                flipped: false
            })
        );
        assert_eq!(
            normalize_logic_jump_range(8, Some(3)),
            Some(LogicJumpRange {
                begin: 3,
                end: 8,
                flipped: true
            })
        );
        assert_eq!(
            normalize_logic_jump_range(5, Some(5)),
            Some(LogicJumpRange {
                begin: 5,
                end: 5,
                flipped: true
            })
        );
        assert_eq!(normalize_logic_jump_range(1, None), None);
        assert_eq!(
            LogicJumpRange::invalid(),
            LogicJumpRange {
                begin: LOGIC_CANVAS_INVALID_JUMP,
                end: LOGIC_CANVAS_INVALID_JUMP,
                flipped: false
            }
        );
    }

    #[test]
    fn logic_canvas_jump_height_assignment_matches_lcanvas_dedup_and_layering() {
        let single = vec![normalize_logic_jump_range(0, Some(3))];
        assert_eq!(
            assign_logic_jump_heights(&single),
            vec![Some(LogicJumpPlacement {
                begin: 0,
                end: 3,
                flipped: false,
                pred_height: 0
            })]
        );

        let disjoint = vec![
            normalize_logic_jump_range(0, Some(2)),
            normalize_logic_jump_range(3, Some(5)),
        ];
        assert_eq!(
            assign_logic_jump_heights(&disjoint)
                .into_iter()
                .map(|placement| placement.unwrap().pred_height)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );

        let nested = vec![
            normalize_logic_jump_range(0, Some(5)),
            normalize_logic_jump_range(1, Some(4)),
        ];
        let nested_heights = assign_logic_jump_heights(&nested)
            .into_iter()
            .map(|placement| placement.unwrap().pred_height)
            .collect::<Vec<_>>();
        assert_eq!(nested_heights, vec![1, 0]);

        let same_forward_end = vec![
            normalize_logic_jump_range(2, Some(6)),
            normalize_logic_jump_range(1, Some(6)),
        ];
        assert_eq!(representative_logic_jumps(&same_forward_end), vec![1]);
        assert_eq!(
            assign_logic_jump_heights(&same_forward_end)
                .into_iter()
                .map(|placement| placement.unwrap().pred_height)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );

        let same_backward_begin = vec![
            normalize_logic_jump_range(6, Some(2)),
            normalize_logic_jump_range(7, Some(2)),
        ];
        assert_eq!(representative_logic_jumps(&same_backward_begin), vec![1]);
        assert_eq!(
            assign_logic_jump_heights(&same_backward_begin)
                .into_iter()
                .map(|placement| placement.unwrap().pred_height)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );

        let mixed = vec![
            normalize_logic_jump_range(0, Some(4)),
            normalize_logic_jump_range(4, Some(0)),
            None,
            normalize_logic_jump_range(1, Some(3)),
        ];
        let placements = assign_logic_jump_heights(&mixed);
        assert_eq!(placements[2], None);
        assert_eq!(placements[0].unwrap().pred_height, 2);
        assert_eq!(placements[1].unwrap().pred_height, 1);
        assert_eq!(placements[3].unwrap().pred_height, 0);
        assert_eq!(placements[1].unwrap().flipped, true);
    }
}
