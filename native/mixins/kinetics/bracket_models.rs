// The bracket's models (Create's, when the look pack is on, and ours), baked beside the block models.
fn kinetics_bracket_models(models: &mut Vec<String>) {
    models.extend(crate::game::kinetics_bracket::all_models());
}
