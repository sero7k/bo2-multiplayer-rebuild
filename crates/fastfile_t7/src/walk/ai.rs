//! AI tables: behavior state machines and trees, animation state machines,
//! selector and mapping tables (HydraX's structs; script strings are u32
//! indices, so only the arrays load). Each: the name, then its arrays in
//! field order, each array's sub-arrays after it.
//!
//! ```text
//! BehaviorStateMachine 0x20: name; 0x08 State* states (0x30: seven u32, 0x20 Connection*, 0x28 u32 count);
//!                      0x10 u32 stateCount; 0x18 State* entry (a reference into the states); Connection 16
//! BehaviorTree 0x28:   name; 0x08 Node* nodes (0x48: four u32, 0x10 u32* children, 0x18 u32 count, ...);
//!                      0x10 u32 nodeCount; 0x18 u32* indices; 0x20 u64 indexCount
//! AnimStateMachine 0x38: name; 0x08 RootState* (0x18: u32 name, 0x08 u32* subStates, 0x10 u32 count), 0x10 count;
//!                      0x18 SubState* (0x40: twelve u32, 0x30 u32* transitions, 0x38 u32 count), 0x20 count;
//!                      0x28 Transition* (0x30, no pointers), 0x30 count
//! AnimSelectorTableSet 0x18: name; 0x08 Selector* (0x58: u32 name, 0x08 Column* (0x18), 0x10 count,
//!                      0x18 Row* (16: Value* (8 each), u32 count), 0x20 count, then u32 column
//!                      indices), 0x10 count
//! AnimMappingTable 0x18: name; 0x08 Map* (0x18: u32 name, 0x08 u32* anims, 0x10 u32 count), 0x10 count
//! ```

use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::stream::Stream;

/// `count` records of `size` at the pointer `field`; then for each, `sub`.
fn each(
    s: &mut Stream,
    field: usize,
    count: usize,
    size: usize,
    skip: &[usize],
    mut sub: impl FnMut(&mut Stream, usize) -> Result<()>,
) -> Result<()> {
    if let Some(t) = s.records(field, count, size, 7)? {
        for i in 0..count {
            s.no_loads(t + size * i, size, skip)?;
            sub(s, t + size * i)?;
        }
    }
    Ok(())
}

fn count(s: &Stream, at: usize) -> Result<usize> {
    Ok(s.u32_at(at)? as usize)
}

pub fn behavior_state_machine(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::BehaviorStateMachine.header_size(), &[0, 8])?;
    let name = s.string(h)?;
    each(s, h + 8, count(s, h + 0x10)?, 0x30, &[0x20], |s, st| {
        s.records(st + 0x20, count(s, st + 0x28)?, 16, 3).map(drop)
    })?;
    Ok(name)
}

pub fn behavior_tree(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::BehaviorTree.header_size(), &[0, 8, 0x18])?;
    let name = s.string(h)?;
    each(s, h + 8, count(s, h + 0x10)?, 0x48, &[0x10], |s, n| s.records(n + 0x10, count(s, n + 0x18)?, 4, 3).map(drop))?;
    s.records(h + 0x18, count(s, h + 0x20)?, 4, 3)?;
    Ok(name)
}

pub fn anim_state_machine(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::AnimStateMachine.header_size(), &[0, 8, 0x18, 0x28])?;
    let name = s.string(h)?;
    each(s, h + 8, count(s, h + 0x10)?, 0x18, &[8], |s, r| s.records(r + 8, count(s, r + 0x10)?, 4, 3).map(drop))?;
    each(s, h + 0x18, count(s, h + 0x20)?, 0x40, &[0x30], |s, r| s.records(r + 0x30, count(s, r + 0x38)?, 4, 3).map(drop))?;
    each(s, h + 0x28, count(s, h + 0x30)?, 0x30, &[], |_, _| Ok(()))?;
    Ok(name)
}

pub fn anim_selector_table_set(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::AnimSelectorTableSet.header_size(), &[0, 8])?;
    let name = s.string(h)?;
    // A selector's words after 0x20 are u32 column indices (-1: none), not pointers.
    each(s, h + 8, count(s, h + 0x10)?, 0x58, &(3..11).map(|i| 8 * i).chain([8]).collect::<Vec<_>>(), |s, sel| {
        s.records(sel + 8, count(s, sel + 0x10)?, 0x18, 7)?;
        each(s, sel + 0x18, count(s, sel + 0x20)?, 16, &[0], |s, row| s.records(row, count(s, row + 8)?, 8, 3).map(drop))
    })?;
    Ok(name)
}

pub fn anim_mapping_table(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::AnimMappingTable.header_size(), &[0, 8])?;
    let name = s.string(h)?;
    each(s, h + 8, count(s, h + 0x10)?, 0x18, &[8], |s, m| s.records(m + 8, count(s, m + 0x10)?, 4, 3).map(drop))?;
    Ok(name)
}
