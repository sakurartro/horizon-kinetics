// The creative motor's models, baked beside the block models (a Create look model missing without its pack is logged).
fn kinetics_motor_models(models: &mut Vec<String>) {
    use crate::game::kinetics_motor as m;
    models.extend(m::MODELS.iter().chain(&m::LOOK_MODELS).map(|s| s.to_string()));
}
