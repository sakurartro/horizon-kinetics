// The creative motor's models, baked beside the block models (a Create look model missing without its pack is logged).
fn kinetics_motor_models(models: &mut Vec<String>) {
    use crate::game::kinetics_motor as m;
    // The half shaft (our and Create's) is registered by the casing's `kinetics_models`, once.
    let shared = crate::game::kinetics_casing::HALF_SHAFT;
    models.extend(m::MODELS.iter().chain(&m::LOOK_MODELS).filter(|s| **s != shared.0 && **s != shared.1).map(|s| s.to_string()));
}
