// Chains the encased chain drives marked in `dimension` this tick to their neighbours and makes each adjustable
// chain gearshift follow its redstone (chain.rs); what changed is solved again.
fn kinetics_chain_settle(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::kinetics_chain as c;
    use horizon_world::nbt::{JavaString, Tag};
    let marks = c::take(dimension);
    if marks.is_empty() {
        return;
    }
    struct W<'a, 'b> {
        game: &'a mut crate::game::Game<'b>,
        dimension: &'a str,
    }
    impl c::World for W<'_, '_> {
        fn block(&self, p: [i32; 3]) -> String {
            self.game.native_block(p)
        }
        fn prop(&self, p: [i32; 3], key: &str) -> Option<String> {
            self.game.native_block_property(p, key)
        }
        fn set_prop(&mut self, p: [i32; 3], key: &str, value: &str) -> bool {
            // Flag 2 is UPDATE_CLIENTS: nothing around it needs to hear.
            self.game.native_set_block_property_with(p, key, value, 2)
        }
        fn signal(&mut self, p: [i32; 3]) -> i32 {
            self.game.native_neighbor_signal(self.dimension, p)
        }
        fn stored_signal(&mut self, p: [i32; 3]) -> i32 {
            let key = JavaString::from(c::SIGNAL_KEY);
            self.game
                .native_with_mod_block_entity(p, |be| match be.nbt.get(&key) {
                    Some(Tag::Int(v)) => *v,
                    _ => 0,
                })
                .unwrap_or(0)
        }
        fn store_signal(&mut self, p: [i32; 3], signal: i32) {
            let key = JavaString::from(c::SIGNAL_KEY);
            self.game.native_with_mod_block_entity(p, |be| be.nbt.insert(key, Tag::Int(signal)));
        }
    }
    let changed = c::settle(&mut W { game: &mut *game, dimension }, &marks);
    let id = match dimension {
        "minecraft:the_nether" => "minecraft:the_nether",
        "minecraft:the_end" => "minecraft:the_end",
        _ => "minecraft:overworld",
    };
    for pos in changed {
        crate::game::kinetics::mark(id, pos);
    }
}
