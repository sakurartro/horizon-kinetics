// The wheels' turning parts, one model of each size per wood, baked beside the block models (and Create's frame).
fn kinetics_water_wheel_models(models: &mut Vec<String>) {
    use crate::game::kinetics_water_wheel as w;
    models.extend(w::models());
    models.push(w::LOOK_FRAME.to_string());
}
