// The gearbox's half shafts, drawn by its block entity.
fn kinetics_gearbox_models(models: &mut Vec<String>) {
    use crate::game::kinetics_gearbox as g;
    // Create's half shaft is registered by the casing's `kinetics_models`, once.
    let shared = crate::game::kinetics_casing::HALF_SHAFT;
    models.extend(g::PART_MODELS.iter().chain(&g::LOOK_MODELS).filter(|m| **m != shared.1).map(|m| m.to_string()));
    models.push(g::STATIC_LOOK.to_string());
}
