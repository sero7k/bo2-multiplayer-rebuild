//! Behavior trees: no x64-zt writer, so the order is inferred from
//! `BehaviorTree` / `BehaviorTreeNode` (x64-zt `iw7/structs.hpp`) and
//! checked against the retail data (`iw7_common_core_mp.ff`'s
//! `seeker_mp`): the name, the node array (24-byte nodes whose only
//! pointer is the name), then each node's name.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

pub(super) fn behavior_tree(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::BehaviorTree as B;
    use L::BehaviorTreeNode as N;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + B::name)?.unwrap_or_default();
    let n = w.s.u16_at(h + B::nodeCount)? as usize;
    if let Some(t) = w.inline_array(h + B::nodes, N::SIZE * n, 7)? {
        for i in 0..n {
            w.xstr(t + N::SIZE * i + N::name)?;
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::BehaviorTree, name))
}
