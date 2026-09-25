//! CR 702.140: mutate, and merged permanents (CR 730).

use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::merge;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

const MUTATE: CastMethod = CastMethod::Keyword(KeywordKind::Mutate);

/// Casts Gemrazer ("Reach, trample; Mutate {1}{G}{G}; Whenever this creature mutates,
/// destroy target artifact or enchantment an opponent controls.") for its mutate cost
/// onto `target`, on top (`on_top`) or under.
fn mutate_gemrazer(t: &mut TestGame, target: ObjectId, on_top: bool) -> ObjectId {
    let gem = t.hand(P0, "Gemrazer");
    add_mana(t, P0, ManaType::G, 2);
    add_mana(t, P0, ManaType::C, 1);
    let spell = t.cast(P0, gem).method(MUTATE).target(target).go();
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(if on_top { 0 } else { 1 }),
    );
    t.resolve();
    spell
}

#[test]
fn a_mutating_spell_merges_with_its_target() {
    cr!("702.140a", "702.140c", "702.140d", "702.140e", "730.2", "730.2a", "730.2b", "730.2c");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    // Giant Growth on the Bears before they mutate: +3/+3 until end of turn.
    let growth = t.hand(P0, "Giant Growth");
    add_mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, growth).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    let spell = mutate_gemrazer(&mut t, bears, true);
    // The Bears are still the same object: it merged, it didn't enter the battlefield
    // (Soul Warden didn't trigger) and it isn't summoning sick.
    assert!(t.g.is_live(bears));
    assert_eq!(t.g.current(spell), bears);
    assert_eq!(t.named_on_battlefield("Gemrazer"), vec![bears]);
    assert!(!t.obj(bears).summoning_sick);
    assert_eq!(merge::physical_components(&t.g, bears).len(), 2);
    // On top: Gemrazer's characteristics, and every component's abilities. The effect
    // that affected the Bears still applies to the merged permanent (CR 730.2c).
    assert_eq!(t.obj(bears).chars.name, "Gemrazer");
    assert_eq!(t.pt(bears), (7, 7));
    assert!(t.obj(bears).has_keyword(KeywordKind::Reach));
    assert!(t.obj(bears).has_keyword(KeywordKind::Trample));
    // "Whenever this creature mutates": the trigger destroys the Sol Ring. It's the only
    // trigger: Soul Warden didn't trigger.
    t.answer_targets(P0, &[Entity::Object(ring)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_mutating_spell_put_under_keeps_the_top_creatures_characteristics() {
    cr!("702.140c", "702.140e", "730.2a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mutate_gemrazer(&mut t, bears, false);
    assert_eq!(t.obj(bears).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    // It has all the abilities of both cards.
    assert!(t.obj(bears).has_keyword(KeywordKind::Reach));
    assert!(t.obj(bears).has_keyword(KeywordKind::Trample));
}

#[test]
fn a_mutating_spell_targets_a_non_human_creature_with_the_same_owner() {
    cr!("702.140a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Not a Human creature (Mardu Hordechief is a Human Warrior), and not a creature
    // owned by the opponent.
    let human = t.battlefield(P0, "Mardu Hordechief");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let gem = t.hand(P0, "Gemrazer");
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 1);
    assert!(t
        .cast(P0, gem)
        .method(MUTATE)
        .target(human)
        .try_go()
        .is_err());
    assert!(t
        .cast(P0, gem)
        .method(MUTATE)
        .target(theirs)
        .try_go()
        .is_err());
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t
        .cast(P0, gem)
        .method(MUTATE)
        .target(bears)
        .try_go()
        .is_ok());
}

#[test]
fn with_an_illegal_target_it_resolves_as_a_creature_spell() {
    cr!("702.140b");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gem = t.hand(P0, "Gemrazer");
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, gem).method(MUTATE).target(bears).go();
    t.g.move_object(bears, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.resolve();
    let gems = t.named_on_battlefield("Gemrazer");
    assert_eq!(gems.len(), 1);
    assert_eq!(t.pt(gems[0]), (4, 4));
    assert!(merge::physical_components(&t.g, gems[0]).is_empty());
}

#[test]
fn a_merged_permanent_leaving_the_battlefield_puts_each_card_there() {
    cr!("730.3");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mutate_gemrazer(&mut t, bears, true);
    t.settle();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    t.g.move_object(bears, Zone::Graveyard(P0), MoveCause::Effect, None);
    // One permanent left the battlefield; both cards are in the graveyard.
    assert!(t.in_graveyard(P0, "Gemrazer"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.graveyard_size(P0), 2);
    assert!(t.g.battlefield.is_empty());
}
