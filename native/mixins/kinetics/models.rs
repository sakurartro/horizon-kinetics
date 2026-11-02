// The turning parts, drawn by the block entities.
fn kinetics_models(models: &mut Vec<String>) {
    use crate::game::kinetics as k;
    // The Create look's models are missing without its pack: the game logs that and draws ours.
    models.extend(k::PART_MODELS.iter().chain(&k::LOOK_MODELS).map(|m| m.to_string()));
}
