// Create's static chain drive and gearshift models, baked beside the block models and drawn by `kinetics_chain_submit`.
fn kinetics_chain_models(models: &mut Vec<String>) {
    use crate::game::kinetics_chain as c;
    models.extend(c::STATIC_LOOK_MODELS.iter().map(|m| m.to_string()));
}
