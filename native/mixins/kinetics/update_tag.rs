// What clients get of a shaft or cogwheel: its speed (and a creative motor's setting, which its value box shows).
fn kinetics_update_tag(entity: &ModBlockEntity, tag: &mut Option<Compound>) {
    use horizon_world::nbt::{JavaString, Tag};
    if !crate::game::kinetics::has_block_entity(&entity.id)
        && !crate::game::kinetics_casing::has_block_entity(&entity.id)
    {
        return;
    }
    let key = JavaString::from(crate::game::kinetics::SPEED_KEY);
    let speed = entity.nbt.get(&key).cloned().unwrap_or(Tag::Float(0f32.to_bits()));
    let mut out = Compound::from([(key, speed)]);
    // A clutch or gearshift has a speed for each half.
    let pos_key = JavaString::from(crate::game::kinetics_clutch::SPEED_POS_KEY);
    if let Some(speed) = entity.nbt.get(&pos_key) {
        out.insert(pos_key, speed.clone());
    }
    let rpm_key = JavaString::from(crate::game::kinetics::RPM_KEY);
    if let Some(rpm) = entity.nbt.get(&rpm_key) {
        out.insert(rpm_key, rpm.clone());
    }
    // A bracket around a shaft or cogwheel.
    let bracket = JavaString::from(crate::game::kinetics_bracket::BRACKET_KEY);
    if let Some(value) = entity.nbt.get(&bracket) {
        out.insert(bracket, value.clone());
    }
    // A water wheel's planks.
    let material = JavaString::from(crate::game::kinetics_water_wheel::MATERIAL_KEY);
    if let Some(value) = entity.nbt.get(&material) {
        out.insert(material, value.clone());
    }
    *tag = Some(out);
}
