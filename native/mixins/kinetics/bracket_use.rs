// A bracket used on a shaft or cogwheel is saved in its block entity (Create's `BracketBlockItem.useOn`): one that
// fits the clicked face replaces the old one, which comes back to the player. Sneaking with the wrench takes the
// bracket off first (`IWrenchableWithBracket`). Brackets are never placed as blocks.
// shortcut: no placing sound, a click on a face along the axis does nothing (Create falls back to the player's look
// direction), the bracket is lost when its shaft is broken, and sneaking is read from the host's keys.
fn kinetics_bracket_use(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    use crate::game::{kinetics as k, kinetics_bracket as b};
    use horizon_world::nbt::{JavaString, Tag};
    if *claimed || !game.may_build() {
        return;
    }
    let item: &str = &stack.item;
    let wrench = item == crate::game::kinetics_wrench::WRENCH;
    let material = b::material_of_item(item);
    if !wrench && material.is_none() {
        return;
    }
    let Some(_) = b::type_of(&game.native_block(pos)) else { return };
    let Some(axis) = game.native_block_property(pos, "axis").as_deref().and_then(k::axis_of) else {
        return;
    };
    let key = JavaString::from(b::BRACKET_KEY);
    let current = game
        .native_with_mod_block_entity(pos, |be| match be.nbt.get(&key) {
            Some(Tag::Int(v)) => b::decode(*v),
            _ => None,
        })
        .flatten();
    let (new, back) = if wrench {
        if !game.keys.contains(&crate::game::KeyCode::ShiftLeft) || current.is_none() {
            return;
        }
        (None, current)
    } else {
        let Some(material) = material else { return };
        let Some(facing) = b::facing_for_click(axis, face) else { return };
        if current == Some((material, facing)) {
            *claimed = true;
            return;
        }
        (Some(b::encode(material, facing)), current)
    };
    game.native_with_mod_block_entity(pos, |be| match new {
        Some(v) => {
            be.nbt.insert(key.clone(), Tag::Int(v));
        }
        None => {
            be.nbt.remove(&key);
        }
    });
    game.native_block_entity_sync(pos);
    if !game.creative {
        if !wrench {
            game.native_take_items(item, 1);
        }
        if let Some((m, _)) = back {
            game.native_give_items(b::ITEMS[m], 1);
        }
    }
    *claimed = true;
}
