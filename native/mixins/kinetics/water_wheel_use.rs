// Planks used on a wheel (or on a large wheel's body) change what it is made of (`applyMaterialIfValid`); nothing is used up.
// shortcut: no break particles, and only vanilla's planks (Create takes any block in its `water_wheel_planks` tag).
fn kinetics_water_wheel_use(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    use crate::game::kinetics_water_wheel as w;
    use horizon_world::nbt::{JavaString, Tag};
    if *claimed || game.keys.contains(&crate::game::KeyCode::ShiftLeft) || !game.may_build() || w::wood_of(&stack.item).is_none() {
        return;
    }
    let name = game.native_block(pos);
    let wheel = if w::is_wheel(&name) {
        pos
    } else if name == w::STRUCTURE {
        let get = |q: [i32; 3]| w::part_of(&game.native_block(q), |key| game.native_block_property(q, key));
        match get(pos) {
            w::Part::Structure(f) if w::still_valid(&get, pos, f, false) => w::master(&get, pos, f),
            _ => return,
        }
    } else {
        return;
    };
    let key = JavaString::from(w::MATERIAL_KEY);
    let item = JavaString::from(&*stack.item);
    let changed = game.native_with_mod_block_entity(wheel, |be| {
        let same = match be.nbt.get(&key) {
            Some(Tag::String(s)) => *s == item,
            _ => &*stack.item == w::DEFAULT_MATERIAL,
        };
        if !same {
            be.nbt.insert(key.clone(), Tag::String(item.clone()));
        }
        !same
    });
    if changed == Some(true) {
        game.native_block_entity_sync(wheel);
        *claimed = true;
    }
}
