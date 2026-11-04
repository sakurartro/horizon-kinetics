// Every block of the content mod has a blockstate, a loot table and a lang name; every recipe result is a block or item of the mod.
use std::{fs, path::Path};

fn names(dir: &str) -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../content").join(dir);
    let mut v: Vec<String> = fs::read_dir(root).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().trim_end_matches(".json").to_string()).collect();
    v.sort();
    v
}

#[test]
fn every_block_has_its_blockstate_loot_and_names() {
    let blocks = names("data/kinetics/horizon/block");
    assert_eq!(blocks, names("assets/kinetics/blockstates"));
    assert_eq!(blocks, names("data/kinetics/loot_table/blocks"));
    for lang in ["en_us", "ru_ru"] {
        let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../content/assets/kinetics/lang/{lang}.json"))).unwrap();
        for b in &blocks {
            assert!(text.contains(&format!("\"block.kinetics.{b}\"")), "{lang} lacks {b}");
        }
    }
}

#[test]
fn recipe_results_exist() {
    let blocks = names("data/kinetics/horizon/block");
    let items = names("data/kinetics/horizon/item");
    for r in names("data/kinetics/recipe") {
        let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../content/data/kinetics/recipe/{r}.json"))).unwrap();
        let id = text.split("\"id\"").nth(1).and_then(|t| t.split('"').nth(1)).unwrap().trim_start_matches("kinetics:").to_string();
        assert!(blocks.contains(&id) || items.contains(&id), "{r} makes unknown {id}");
    }
}
