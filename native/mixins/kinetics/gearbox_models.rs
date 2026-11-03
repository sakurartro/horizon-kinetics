// The gearbox's half shafts, drawn by its block entity.
fn kinetics_gearbox_models(models: &mut Vec<String>) {
    use crate::game::kinetics_gearbox as g;
    models.extend(g::PART_MODELS.iter().chain(&g::LOOK_MODELS).map(|m| m.to_string()));
}
