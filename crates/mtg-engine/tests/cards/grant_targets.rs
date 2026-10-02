//! Target and group qualifiers of grants and pumps (`src/oracle/phrases.rs`,
//! `src/oracle/patterns/grant_grammar.rs`): "1/1 creature" (its current power and
//! toughness), "nonattacking, nonblocking", "that's attacking alone" (CR 506.5), "that
//! dealt damage this turn"; a player gaining shroud (CR 702.18a); and "~ deals ... and
//! gains ..." (one subject, two predicates).

use mtg_engine::eval::Ctx;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The legal choices for the first target of `source`'s `index`th activated ability.
fn ability_targets(t: &mut TestGame, source: ObjectId, index: usize) -> Vec<Entity> {
    t.g.recompute();
    let o = t.g.obj(source);
    let spec = o
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            ability::AbilityKind::Activated(x) => Some(x),
            _ => None,
        })
        .nth(index)
        .and_then(|a| a.body.targets.first().cloned())
        .expect("no targeted activated ability");
    let ctx = Ctx::new(Some(source), o.controller);
    t.g.legal_target_candidates(&spec, &ctx, source)
}

/// The legal choices for the first target of the spell `name` cast by `p`.
fn spell_targets(t: &mut TestGame, p: PlayerId, name: &str) -> Vec<Entity> {
    let spell = t.hand(p, name);
    t.g.recompute();
    let spec = t.g.spell_body(spell).targets[0].clone();
    let ctx = Ctx::new(Some(spell), p);
    t.g.legal_target_candidates(&spec, &ctx, spell)
}

#[test]
fn pendelhaven_targets_only_creatures_that_are_1_1_now() {
    cr!("115.1", "208.1");
    ruling!(
        "Pendelhaven",
        "The current power/toughness being 1/1 is a targeting requirement"
    );
    assert_supported(&["Pendelhaven", "Pendelhaven Elder"]);
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Pendelhaven");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pumped = t.battlefield(P1, "Llanowar Elves");
    t.g.add_counters(Entity::Object(pumped), "+1/+1", 1, None);
    let targets = ability_targets(&mut t, ph, 1);
    assert!(targets.contains(&Entity::Object(elf)));
    assert!(!targets.contains(&Entity::Object(bears)));
    assert!(!targets.contains(&Entity::Object(pumped)), "it's 2/2 now");
    // Pendelhaven Elder: each 1/1 creature you control.
    let elder = t.battlefield(P0, "Pendelhaven Elder");
    t.activate(P0, elder, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(elder), (2, 3));
    assert_eq!(t.pt(elf), (2, 3));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(pumped), (2, 2));
}

#[test]
fn viper_targets_a_creature_attacking_alone() {
    cr!("506.5");
    assert_supported(&["Viper, Cruel Conspirator", "Unlikely Alliance"]);
    let mut t = TestGame::new(2);
    let viper = t.battlefield(P0, "Viper, Cruel Conspirator");
    let ua = t.battlefield(P0, "Unlikely Alliance");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let idle = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        mtg_engine::testing::DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(a, Entity::Player(P0))]),
    );
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareAttackers
            && g.turn.stage == mtg_engine::turn::Stage::Priority
    });
    assert!(ok);
    let targets = ability_targets(&mut t, viper, 0);
    assert_eq!(targets, vec![Entity::Object(a)]);
    // Unlikely Alliance: nonattacking, nonblocking.
    let targets = ability_targets(&mut t, ua, 0);
    assert!(targets.contains(&Entity::Object(b)));
    assert!(targets.contains(&Entity::Object(idle)));
    assert!(!targets.contains(&Entity::Object(a)));
}

#[test]
fn executioners_swing_targets_only_creatures_that_dealt_damage() {
    cr!("115.1", "120.1");
    ruling!(
        "Executioner's Swing",
        "It doesn’t matter what the creature dealt damage to or if that damage was combat damage"
    );
    assert_supported(&["Executioner's Swing"]);
    let mut t = TestGame::new(2);
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!spell_targets(&mut t, P0, "Executioner's Swing").contains(&Entity::Object(pinger)));
    // Noncombat damage to a player counts.
    t.activate(P1, pinger, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    let targets = spell_targets(&mut t, P0, "Executioner's Swing");
    assert!(targets.contains(&Entity::Object(pinger)));
    assert!(!targets.contains(&Entity::Object(bears)));
}

#[test]
fn gilded_light_you_gain_shroud() {
    cr!("702.18a");
    assert_supported(&["Gilded Light"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Gilded Light");
    t.cast(P0, s).go();
    t.resolve();
    // Neither an opponent nor you can target you.
    assert!(!spell_targets(&mut t, P1, "Lightning Bolt").contains(&Entity::Player(P0)));
    assert!(!spell_targets(&mut t, P0, "Lightning Bolt").contains(&Entity::Player(P0)));
    assert!(spell_targets(&mut t, P1, "Lightning Bolt").contains(&Entity::Player(P1)));
}

#[test]
fn ellie_deals_damage_and_gains_indestructible() {
    cr!("608.2c");
    assert_supported(&["Ellie, Vengeful Hunter"]);
    let mut t = TestGame::new(2);
    let ellie = t.battlefield(P0, "Ellie, Vengeful Hunter");
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, ellie, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 18);
    assert!(t.obj_now(ellie).has_keyword(KeywordKind::Indestructible));
    assert!(!t.in_graveyard(P0, "Ellie, Vengeful Hunter"));
}
