//! Commander rules in Oracle text (CR 903): "commander" as a noun and an adjective
//! ("target commander", "commander creatures", "except for commanders"), the command zone
//! as a zone ("from the command zone", "to the command zone"), an ability that functions
//! in the command zone, a flashback cost reduction "this way", "your commander" in a
//! mana restriction, and commander casts counted for another player.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::game::Variant;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn commander_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

/// Makes `id` one of `p`'s commanders (CR 903.3).
fn make_commander(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let name = t.g.obj(id).chars.name.clone();
    t.g.players[p.idx()].commander_names.push(name);
    t.g.recompute();
}

fn commander_in_zone(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.command(p, name);
    make_commander(t, p, id);
    id
}

fn commander_on_battlefield(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    make_commander(t, p, id);
    id
}

#[test]
fn slash_the_ranks_spares_commanders() {
    cr!("903.3");
    assert_supported(&["Slash the Ranks"]);
    let mut t = commander_game();
    let cmdr = commander_on_battlefield(&mut t, P1, "Esior, Wardwing Familiar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    let s = t.hand(P0, "Slash the Ranks");
    t.cast(P0, s).go();
    t.resolve();
    assert!(t.on_battlefield(cmdr));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn gray_harbor_merfolk_counts_a_commander_creature_you_control() {
    cr!("903.3");
    assert_supported(&["Gray Harbor Merfolk"]);
    let mut t = commander_game();
    let m = t.battlefield(P0, "Gray Harbor Merfolk");
    assert_eq!(t.pt(m), (0, 3));
    // An opponent's commander doesn't count.
    commander_on_battlefield(&mut t, P1, "Esior, Wardwing Familiar");
    assert_eq!(t.pt(m), (0, 3));
    commander_on_battlefield(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(m), (2, 3));
}

#[test]
fn witchs_clinic_targets_only_commanders() {
    cr!("903.3", "115.1");
    assert_supported(&["Witch's Clinic"]);
    let mut t = commander_game();
    let clinic = t.battlefield(P0, "Witch's Clinic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cmdr = commander_on_battlefield(&mut t, P0, "Esior, Wardwing Familiar");
    t.lands(P0, "Plains", 2);
    t.activate(P0, clinic, 1, &[Entity::Object(cmdr)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(cmdr).chars.has_keyword(KeywordKind::Lifelink));
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Lifelink));
}

#[test]
fn sanctum_of_eternity_returns_a_commander_from_the_battlefield() {
    cr!("903.3", "903.9b");
    assert_supported(&["Sanctum of Eternity"]);
    let mut t = commander_game();
    let s = t.battlefield(P0, "Sanctum of Eternity");
    let cmdr = commander_on_battlefield(&mut t, P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    // CR 903.9b: the owner may put it into the command zone instead; they don't.
    t.answer_yes(P0, false);
    t.activate(P0, s, 1, &[Entity::Object(cmdr)]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn leadership_vacuum_returns_commanders_to_the_command_zone() {
    cr!("903.3", "408.1");
    assert_supported(&["Leadership Vacuum"]);
    let mut t = commander_game();
    let cmdr = commander_on_battlefield(&mut t, P1, "Esior, Wardwing Familiar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let lv = t.hand(P0, "Leadership Vacuum");
    t.cast(P0, lv).target(P1).go();
    t.resolve();
    assert_eq!(t.zone(t.g.current(cmdr)), Zone::Command);
    assert!(t.on_battlefield(bears));
}

#[test]
fn hellkite_courser_puts_a_commander_onto_the_battlefield_until_end_step() {
    cr!("903.3", "603.7");
    assert_supported(&["Hellkite Courser"]);
    let mut t = commander_game();
    let cmdr = commander_in_zone(&mut t, P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.enter(P0, "Hellkite Courser");
    t.resolve_all();
    let on_bf = t.g.current(cmdr);
    assert!(t.on_battlefield(on_bf));
    assert!(t.obj_now(on_bf).chars.has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(cmdr)), Zone::Command);
}

#[test]
fn campfire_puts_commanders_from_the_command_zone_and_graveyard_into_hand() {
    cr!("903.3", "903.9b");
    assert_supported(&["Campfire"]);
    let mut t = commander_game();
    commander_in_zone(&mut t, P0, "Esior, Wardwing Familiar");
    let g = t.graveyard(P0, "Grizzly Bears");
    make_commander(&mut t, P0, g);
    t.graveyard(P0, "Hill Giant");
    let c = t.battlefield(P0, "Campfire");
    t.lands(P0, "Mountain", 2);
    // CR 903.9b: the owner may put them into the command zone instead; they don't.
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.activate(P0, c, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Esior, Wardwing Familiar"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Hill Giant"));
}

#[test]
fn derevi_enters_from_the_command_zone() {
    cr!("113.6", "903.3");
    assert_supported(&["Derevi, Empyrial Tactician"]);
    let mut t = commander_game();
    let d = commander_in_zone(&mut t, P0, "Derevi, Empyrial Tactician");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Derevi, Empyrial Tactician").len(), 1);
}

#[test]
fn commanders_insight_counts_the_target_players_commander_casts() {
    cr!("903.8");
    assert_supported(&["Commander's Insight"]);
    let mut t = commander_game();
    commander_in_zone(&mut t, P1, "Esior, Wardwing Familiar");
    t.g.players[1]
        .commander_casts
        .insert("Esior, Wardwing Familiar".into(), 2);
    t.lands(P0, "Island", 4);
    let ci = t.hand(P0, "Commander's Insight");
    let before = t.hand_size(P1);
    t.cast(P0, ci).x(1).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), before + 3);
}

#[test]
fn visions_of_dread_costs_less_only_with_flashback() {
    cr!("702.34a", "601.2f");
    assert_supported(&["Visions of Dread", "Visions of Glory"]);
    let mut t = commander_game();
    // A commander with mana value 4 in the command zone: flashback {8}{B}{B} costs
    // {4}{B}{B}.
    commander_in_zone(&mut t, P0, "Hill Giant");
    t.graveyard(P1, "Grizzly Bears");
    let v = t.graveyard(P0, "Visions of Dread");
    t.lands(P0, "Swamp", 5);
    let r = t
        .cast(P0, v)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(P1)
        .try_go();
    assert!(r.is_err());
    t.lands(P0, "Swamp", 1);
    t.cast(P0, v)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(P1)
        .go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_exile("Visions of Dread"));
}

#[test]
fn jeweled_lotus_mana_only_casts_your_commander() {
    cr!("106.6", "903.3");
    assert_supported(&["Jeweled Lotus"]);
    let mut t = commander_game();
    let lotus = t.battlefield(P0, "Jeweled Lotus");
    let cmdr = commander_in_zone(&mut t, P0, "Hill Giant");
    let other = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, lotus, 0, &[]).unwrap();
    // The three mana can't pay for another spell.
    assert!(t.cast(P0, other).try_go().is_err());
    t.cast(P0, cmdr).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn esior_taxes_spells_targeting_your_commanders() {
    cr!("601.2f", "903.3");
    assert_supported(&["Esior, Wardwing Familiar"]);
    let mut t = commander_game();
    let esior = commander_on_battlefield(&mut t, P0, "Esior, Wardwing Familiar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    // Bolting a noncommander costs {R}; bolting the commander costs {3}{R}.
    assert!(t.cast(P1, bolt).target(esior).try_go().is_err());
    t.cast(P1, bolt).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
}
