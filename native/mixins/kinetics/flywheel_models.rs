// The flywheel's wheel, baked beside the block models.
fn kinetics_flywheel_models(models: &mut Vec<String>) {
    models.push(crate::game::kinetics_flywheel::WHEEL_MODEL.to_string());
}
