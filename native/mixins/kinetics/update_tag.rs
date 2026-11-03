// What clients get of a shaft or cogwheel: its speed.
fn kinetics_update_tag(entity: &ModBlockEntity, tag: &mut Option<Compound>) {
    use horizon_world::nbt::{JavaString, Tag};
    if !crate::game::kinetics::has_block_entity(&entity.id)
        && !crate::game::kinetics_casing::has_block_entity(&entity.id)
    {
        return;
    }
    let key = JavaString::from(crate::game::kinetics::SPEED_KEY);
    let speed = entity.nbt.get(&key).cloned().unwrap_or(Tag::Float(0f32.to_bits()));
    *tag = Some(Compound::from([(key, speed)]));
}
