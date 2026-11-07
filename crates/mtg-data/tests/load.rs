use std::time::Instant;

#[test]
fn loads_all_data() {
    let t = Instant::now();
    let rules = mtg_data::comprehensive_rules();
    eprintln!("CR loaded in {:?}", t.elapsed());
    assert_eq!(rules.effective_date, "September 25, 2026");
    let n = rules.numbered_rules().count();
    eprintln!("numbered rules: {n}, glossary: {}", rules.glossary.len());
    assert!(n > 3000, "{n}");
    assert!(rules.get("704.5a").unwrap().text.contains("0 or less life"));
    assert!(rules.get("702.19").is_some());
    assert_eq!(
        rules.get("704.5aa").unwrap().parent.as_deref(),
        Some("704.5")
    );
    assert!(rules.glossary_entry("Abandon").is_some());

    let t = Instant::now();
    let cards = mtg_data::cards();
    eprintln!("cards loaded in {:?}: {}", t.elapsed(), cards.len());
    let bolt = cards.by_name("Lightning Bolt").unwrap();
    assert_eq!(bolt.mana_cost.as_deref(), Some("{R}"));
    assert!(cards.by_name("Fire").is_some());
    assert!(cards.by_name("fire // ice").is_some());

    let t = Instant::now();
    let rulings = mtg_data::rulings();
    eprintln!("rulings loaded in {:?}: {}", t.elapsed(), rulings.len());
    let r = mtg_data::ruling!("Tarmogoyf", "card types");
    assert!(!r.is_empty());
    assert_eq!(rules.numbered_rules().count(), 3166);
}

#[test]
fn cr_macro_accepts_real_rules() {
    mtg_data::cr!("100.1", "704.5a", "510.1c");
}

#[test]
#[should_panic]
fn cr_macro_rejects_fake_rules() {
    mtg_data::assert_rule_exists(&format!("704.5{}", "zz"));
}

#[test]
#[ignore]
fn dump_ids() {
    let ids: Vec<String> = mtg_data::comprehensive_rules()
        .numbered_rules()
        .map(|r| r.id.clone())
        .collect();
    std::fs::write(std::env::var("DUMP").unwrap(), ids.join("\n")).unwrap();
}
