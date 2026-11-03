// The clocks' models, baked beside the block models (a Create look model missing without its pack is logged).
fn kinetics_clock_models(models: &mut Vec<String>) {
    use crate::game::kinetics_clock as c;
    // The half shaft is registered by the casing's `kinetics_models`, once.
    models.extend(c::PART_MODELS.iter().chain(&c::LOOK_MODELS).map(|m| m.to_string()));
}
