//! Durations of granted abilities (`src/oracle/patterns/grant_grammar.rs`): "for as long
//! as that creature has a [kind] counter on it, it has \"...\"" (CR 611.2b) and "until
//! this card is cast from exile" with the permission to cast it while it remains exiled
//! (CR 400.7j, 601.2a).

use mtg_engine::ability::AbilityKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn count_kind(t: &TestGame, id: ObjectId, pred: fn(&AbilityKind) -> bool) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| pred(&a.kind))
        .count()
}

fn triggered(k: &AbilityKind) -> bool {
    matches!(k, AbilityKind::Triggered(_))
}

fn activated(k: &AbilityKind) -> bool {
    matches!(k, AbilityKind::Activated(_))
}

#[test]
fn mathas_bounty_ability_lasts_while_the_counter_is_there() {
    cr!("611.2b", "113.7");
    ruling!(
        "Mathas, Fiend Seeker",
        "still has the triggered ability it gains even if Mathas leaves the battlefield"
    );
    ruling!(
        "Mathas, Fiend Seeker",
        "are the opponents of the player who controlled the creature as it died"
    );
    assert_supported(&["Mathas, Fiend Seeker"]);
    let mut t = TestGame::new(2);
    let mathas = t.battlefield(P0, "Mathas, Fiend Seeker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(bears, "bounty"), 1);
    assert_eq!(count_kind(&t, bears, triggered), 1);
    // Mathas leaving doesn't end it.
    t.g.destroy_all(vec![mathas], None, false);
    t.settle();
    assert_eq!(count_kind(&t, bears, triggered), 1);
    // The Bears die: P0 (its controller's opponent) draws and gains 2 life.
    let hand = t.hand_size(P0);
    t.g.destroy_all(vec![bears], None, false);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn mathas_ability_ends_when_the_counter_is_removed() {
    cr!("611.2b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mathas, Fiend Seeker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(count_kind(&t, bears, triggered), 1);
    t.g.remove_counters(Entity::Object(bears), "bounty", 1);
    t.settle();
    assert_eq!(count_kind(&t, bears, triggered), 0);
    // Putting a bounty counter on it again doesn't bring the ability back.
    t.g.add_counters(Entity::Object(bears), "bounty", 1, None);
    t.settle();
    assert_eq!(count_kind(&t, bears, triggered), 0);
}

#[test]
fn makeshift_mannequin_returned_creature_is_sacrificed_when_targeted() {
    cr!("611.2b", "603.2");
    assert_supported(&["Makeshift Mannequin"]);
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let s = t.hand(P0, "Makeshift Mannequin");
    t.cast(P0, s).target(gy).go();
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(bears, "mannequin"), 1);
    // Targeted by an opponent's Shock: the trigger sacrifices it first.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn masked_bandits_grant_lasts_until_the_card_is_cast_from_exile() {
    cr!("400.7j", "611.2b", "601.2a");
    ruling!(
        "Masked Bandits",
        "You may use the mana ability Masked Bandits grants to the land while casting"
    );
    assert_supported(&["Masked Bandits"]);
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Wastes");
    t.lands(P0, "Swamp", 2);
    let mb = t.hand(P0, "Masked Bandits");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, mb, 0, &[Entity::Object(land)]).unwrap();
    t.resolve();
    assert_eq!(count_kind(&t, land, activated), 2);
    let exiled = t.g.current(mb);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // Cast it from exile (the land's new ability helps pay for it).
    t.g.untap(land);
    for id in t.g.battlefield.clone() {
        t.g.untap(id);
    }
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.cast(P0, exiled).go();
    t.settle();
    assert_eq!(count_kind(&t, land, activated), 1, "the effect ended");
    t.resolve();
    assert_eq!(t.named_on_battlefield("Masked Bandits").len(), 1);
}

#[test]
fn masked_bandits_grant_stays_if_the_card_leaves_exile_otherwise() {
    cr!("611.2b");
    ruling!(
        "Masked Bandits",
        "If Masked Bandits is removed from exile without being cast"
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Wastes");
    t.lands(P0, "Swamp", 2);
    let mb = t.hand(P0, "Masked Bandits");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, mb, 0, &[Entity::Object(land)]).unwrap();
    t.resolve();
    let exiled = t.g.current(mb);
    t.g.move_object(exiled, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
    t.settle();
    assert_eq!(count_kind(&t, land, activated), 2);
}

/// Obsidian Fireheart: the land keeps burning after the Fireheart leaves (the duration is
/// about the land's counter, CR 611.2b), "this land" is the land and "you" its
/// controller, and it stops once the counter is gone.
#[test]
fn obsidian_fireheart_land_burns_while_it_has_a_blaze_counter() {
    cr!("611.2b", "113.6");
    assert_supported(&["Obsidian Fireheart"]);
    let mut t = TestGame::new(2);
    let fh = t.battlefield(P0, "Obsidian Fireheart");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Mountain", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, fh, 0, &[Entity::Object(land)]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(land).counters.get("blaze").copied(), Some(1));
    t.g.destroy_all(vec![fh], None, false);
    t.settle();
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
    // Without the counter, no more ability.
    t.g.objects[land.0 as usize].counters.remove("blaze");
    t.g.recompute();
    assert_eq!(count_kind(&t, land, triggered), 0);
}
