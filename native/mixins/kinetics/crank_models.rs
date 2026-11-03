// The crank's turning part, baked beside the block models (with Create's, when the look pack is on).
fn kinetics_crank_models(models: &mut Vec<String>) {
    use crate::game::kinetics_crank as c;
    // The half shaft (Create's) is registered by the casing's `kinetics_models`, once.
    let shared = crate::game::kinetics_casing::HALF_SHAFT;
    models.extend(std::iter::once(c::PART_MODEL).chain(c::LOOK_MODELS).filter(|m| *m != shared.0 && *m != shared.1).map(|m| m.to_string()));
}
