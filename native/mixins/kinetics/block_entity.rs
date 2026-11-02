// A shaft or cogwheel gets a block entity, its speed saved in it.
fn kinetics_block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && crate::game::kinetics::has_block_entity(block) {
        *entity = Some((block.to_string(), Compound::new()));
    }
}
