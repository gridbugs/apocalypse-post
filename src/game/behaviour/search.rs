use game::*;
use ecs_content::Entity;

use behaviour::{LeafResolution, SwitchResolution};
use search::{GridSearchCfg, GridSearchCtx};

pub fn follow_path_step<K: KnowledgeRenderer>() -> BehaviourLeaf<K> {
    BehaviourLeaf::new(move |input| {
        let mut path_traverse = input.entity.borrow_mut_path_traverse().unwrap();
        let action = if let Some(direction) = path_traverse.next_direction() {
            MetaAction::ActionArgs(ActionArgs::Walk(input.entity.id(), direction))
        } else {
            MetaAction::ActionArgs(ActionArgs::Null)
        };
        LeafResolution::Yield(action)
    })
}

pub fn simple_npc_update_path<K: KnowledgeRenderer>() -> BehaviourLeaf<K> {
    let search_ctx = GridSearchCtx::new();
    let search_cfg = GridSearchCfg::cardinal_directions();

    BehaviourLeaf::new(move |input| {
        let position = input.entity.copy_position().unwrap();
        let knowledge = input.entity.borrow_simple_npc_knowledge().unwrap();
        let level_knowledge = knowledge.level(input.level_id);
        let mut path_traverse = input.entity.borrow_mut_path_traverse().unwrap();

        let result = search_ctx.search_predicate(
            level_knowledge.grid(), position,
            |info| level_knowledge.contains_target(info.coord),
            &search_cfg, path_traverse.path_mut());

        if result.is_err() {
            return LeafResolution::Return(false);
        }

        path_traverse.reset();
        LeafResolution::Return(true)
    })
}

pub fn simple_npc_coherence<K: KnowledgeRenderer>(child: BehaviourNodeIndex) -> BehaviourSwitch<K> {
    BehaviourSwitch::new_returning(move |input| {
        let path_traverse = input.entity.borrow_mut_path_traverse().unwrap();
        if path_traverse.is_complete() {
            return SwitchResolution::Reset(child);
        } else {
            return SwitchResolution::Select(child);
        }
    })
}
