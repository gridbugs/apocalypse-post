use ecs::*;
use spatial_hash::*;
use util::LeakyReserver;
use math::Coord;

struct Env {
    sh: SpatialHashTable,
    ctx: EcsCtx,
    ids: LeakyReserver<EntityId>,
}

impl Env {
    fn new() -> Self {
        Env {
            sh: SpatialHashTable::new(),
            ctx: EcsCtx::new(),
            ids: LeakyReserver::new(),
        }
    }
}

#[test]
fn insert_remove() {
    let mut env = Env::new();

    let mut action = EcsAction::new();
    let coord = Coord::new(1, 2);

    let id = {
        let mut entity = action.entity_mut(env.ids.reserve());
        entity.insert_position(coord);
        entity.insert_solid();

        entity.id()
    };

    assert!(!env.sh.get(coord).solid());
    env.sh.update(&env.ctx, &action);
    env.ctx.commit(&mut action); // this resets the action so it can be reused
    assert!(env.sh.get(coord).solid());

    {
        let mut entity = action.entity_mut(id);
        entity.remove_solid();
    }

    env.sh.update(&env.ctx, &action);
    env.ctx.commit(&mut action);
    assert!(!env.sh.get(coord).solid());
}

#[test]
fn insert_move() {
    let mut env = Env::new();

    let mut action = EcsAction::new();
    let start_coord = Coord::new(1, 2);
    let end_coord = Coord::new(1, 3);

    let id = {
        let mut entity = action.entity_mut(env.ids.reserve());
        entity.insert_position(start_coord);
        entity.insert_solid();

        entity.id()
    };

    env.sh.update(&env.ctx, &action);
    env.ctx.commit(&mut action);

    {
        let mut entity = action.entity_mut(id);
        entity.insert_position(end_coord);
    }

    env.sh.update(&env.ctx, &action);
    env.ctx.commit(&mut action);

    assert!(!env.sh.get(start_coord).solid());
    assert!(env.sh.get(end_coord).solid());
}
