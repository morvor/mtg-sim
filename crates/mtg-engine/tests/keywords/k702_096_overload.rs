//! CR 702.96 Overload.

use crate::common_k702_011_017::{assert_supported, cast_targets};
use crate::common_k702_027_037::can_cast;
use crate::common_k702_052_066::run_effect;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

const OVERLOAD: CastMethod = CastMethod::Keyword(KeywordKind::Overload);

fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.is_land() && !o.tapped)
        .count()
}

fn gain_keyword(t: &mut TestGame, id: ObjectId, kw: Keyword) {
    run_effect(
        t,
        None,
        t.g.obj(id).controller,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(kw)],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn overload_is_an_alternative_cost_that_changes_target_to_each() {
    cr!("702.96", "702.96a", "702.96b");
    ruling!(
        "Cyclonic Rift",
        "If you don't pay the overload cost of a spell with overload, that spell will have a single target. If you pay the overload cost, the spell won't have any targets."
    );
    assert_supported("Cyclonic Rift");
    // Cast normally: one target.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Island", 2);
    let rift = t.hand(P0, "Cyclonic Rift");
    t.cast(P0, rift).target(a).go();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(b));
    // Overloaded: each nonland permanent you don't control, no targets.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Island", 7);
    let rift = t.hand(P0, "Cyclonic Rift");
    let from = t.asked().len();
    let spell = t.cast(P0, rift).method(OVERLOAD).go();
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        mtg_engine::decision::Decision::ChooseTargets { .. }
    )));
    assert!(t.g.obj(spell).stack.as_ref().unwrap().chosen.iter().all(|c| c.targets.iter().all(|v| v.is_empty())));
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(t.on_battlefield(land));
    assert!(t.on_battlefield(mine));
    let _ = (a, b);
}

#[test]
fn an_overloaded_spell_affects_what_it_couldnt_target() {
    cr!("702.96b");
    ruling!(
        "Cyclonic Rift",
        "Because a spell with overload doesn't target when its overload cost is paid, it may affect permanents with hexproof or with protection from the appropriate color."
    );
    ruling!(
        "Mizzium Mortars",
        "Note that if the spell with overload is dealing damage, protection from that spell's color will still prevent that damage."
    );
    let mut t = TestGame::new(2);
    let hexproof = t.battlefield(P1, "Grizzly Bears");
    gain_keyword(&mut t, hexproof, Keyword::new(KeywordKind::Hexproof));
    let pro_red = t.battlefield(P1, "Hill Giant");
    gain_keyword(
        &mut t,
        pro_red,
        Keyword::with_filter(KeywordKind::Protection, Filter::Color(Color::Red)),
    );
    let mine = t.battlefield(P0, "Llanowar Elves");
    // Not a legal target for the normal cast.
    t.lands(P0, "Mountain", 6);
    let mortars = t.hand(P0, "Mizzium Mortars");
    let targets = cast_targets(&mut t, P0, mortars);
    assert!(!targets.contains(&Entity::Object(hexproof)));
    assert!(!targets.contains(&Entity::Object(pro_red)));
    // Overloaded: 4 damage to each creature you don't control.
    let mortars = t.g.current(mortars);
    t.cast(P0, mortars).method(OVERLOAD).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Protection from red still prevents the damage.
    assert!(t.on_battlefield(pro_red));
    assert_eq!(t.g.obj(pro_red).damage, 0);
    assert!(t.on_battlefield(mine));
}

#[test]
fn overload_changes_the_spells_text() {
    cr!("702.96c", "612.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let rift = t.hand(P0, "Cyclonic Rift");
    let spell = t.cast(P0, rift).method(OVERLOAD).go();
    let o = t.g.obj(spell);
    assert!(o.chars.rules_text.contains("Return each nonland permanent"));
    assert!(!o.chars.rules_text.contains("target"));
    let spell_ability = o
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Spell(_)))
        .unwrap();
    assert!(spell_ability.text.contains("each nonland permanent"));
    // The card's own text isn't changed: only the spell cast for its overload cost.
    t.resolve();
    let card = t.g.current(spell);
    assert!(t.g.obj(card).chars.rules_text.contains("target"));
}

#[test]
fn overloading_doesnt_change_mana_value_and_cost_increases_apply() {
    cr!("702.96a", "601.2f");
    ruling!(
        "Cyclonic Rift",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as an overload cost), add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 7);
    let rift = t.hand(P0, "Cyclonic Rift");
    // {6}{U} + {1}: seven lands aren't enough.
    assert!(t.cast(P0, rift).method(OVERLOAD).try_go().is_err());
    t.lands(P0, "Island", 1);
    let spell = t.cast(P0, rift).method(OVERLOAD).go();
    assert_eq!(t.g.obj(spell).chars.mana_value(), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn a_spell_cast_without_paying_its_mana_cost_cant_be_overloaded() {
    cr!("702.96a");
    ruling!(
        "Cyclonic Rift",
        "If you are instructed to cast a spell with overload \"without paying its mana cost,\" you can't choose to pay its overload cost instead."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let rift = t.exile(P0, "Cyclonic Rift");
    t.answer_targets(P0, &[Entity::Object(a)]);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(rift)],
    );
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(b));
    // Nor can a card be overloaded from a zone it couldn't be cast from.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let in_gy = t.graveyard(P0, "Cyclonic Rift");
    assert!(!can_cast(&mut t, P0, in_gy, OVERLOAD));
    let in_hand = t.hand(P0, "Cyclonic Rift");
    assert!(can_cast(&mut t, P0, in_hand, OVERLOAD));
}

#[test]
fn an_overloaded_spell_affects_what_is_there_as_it_resolves() {
    cr!("702.96a", "702.96b");
    ruling!(
        "Dynacharge",
        "An overloaded Dynacharge affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +2/+0."
    );
    assert_supported("Dynacharge");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let dyna = t.hand(P0, "Dynacharge");
    t.cast(P0, dyna).method(OVERLOAD).go();
    // A creature that enters before it resolves is affected.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.resolve();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(elves), (3, 1));
    assert_eq!(t.pt(theirs), (2, 2));
    // One that enters later isn't.
    let giant = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn target_player_becomes_each_player() {
    cr!("702.96a");
    assert_supported("Mind Rake");
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        for _ in 0..3 {
            t.hand(p, "Grizzly Bears");
        }
    }
    t.lands(P0, "Swamp", 2);
    let rake = t.hand(P0, "Mind Rake");
    t.cast(P0, rake).method(OVERLOAD).go();
    t.resolve();
    // "Each player discards two cards."
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.hand_size(P2), 1);
    assert_eq!(t.zone(rake), Zone::Graveyard(P0));
}

#[test]
fn a_copy_of_an_overloaded_spell_is_overloaded() {
    cr!("702.96c", "707.10");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let dyna = t.hand(P0, "Dynacharge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let spell = t.cast(P0, dyna).method(OVERLOAD).go();
    run_effect(
        &mut t,
        None,
        P0,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(spell)],
    );
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    assert!(t.g.obj(copy).chars.rules_text.contains("Each creature"));
    t.resolve_all();
    // Each creature you control got +2/+0 twice.
    assert_eq!(t.pt(bears), (6, 2));
    assert_eq!(t.pt(elves), (5, 1));
}
