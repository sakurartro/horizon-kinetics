// The turning parts, drawn by the block entities.
fn kinetics_models(models: &mut Vec<String>) {
    models.extend(crate::game::kinetics::PART_MODELS.iter().map(|m| m.to_string()));
}
