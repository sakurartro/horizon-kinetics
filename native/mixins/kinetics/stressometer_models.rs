// Create's stressometer models (bodies, dial, head), baked beside the block models; missing without its pack: logged, not drawn.
fn kinetics_stressometer_models(models: &mut Vec<String>) {
    models.extend(crate::game::kinetics_stressometer::LOOK_MODELS.iter().map(|m| m.to_string()));
}
