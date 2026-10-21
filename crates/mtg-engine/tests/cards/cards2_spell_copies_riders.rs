//! Riders on spell copies (CR 707.10, 707.10f, 115.7d): "those spells gain wither"
//! (Spinerock Tyrant), "If the copy is a permanent spell, it gains haste ..." (Mizzix,
//! Replica Rider; Spider-Verse), and a spell's own ability choosing a new target for it
//! (Lightning Storm).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn spinerock_tyrant_copies_and_both_spells_gain_wither() {
    cr!("707.10", "702.80a");
    supported("Spinerock Tyrant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spinerock Tyrant");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(a).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(b)]);
    t.resolve();
    assert!(t.g.obj(spell).has_keyword(KeywordKind::Wither));
    t.resolve_all();
    // Damage from sources with wither is dealt as -1/-1 counters.
    assert_eq!(t.counters(a, "-1/-1"), 3);
    assert_eq!(t.obj_now(a).damage, 0);
    // The copy's three -1/-1 counters killed the 3/3.
    assert!(!t.on_battlefield(b));
}

#[test]
fn mizzix_copy_of_a_permanent_spell_gains_haste_and_is_sacrificed() {
    cr!("707.10f", "707.10");
    supported("Mizzix, Replica Rider");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mizzix, Replica Rider");
    t.battlefield(P0, "Walking Corpse");
    let gc = t.graveyard(P0, "Gravecrawler");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Island", 2);
    t.answer_yes(P0, true);
    t.cast(P0, gc).go();
    t.resolve_all();
    let crawlers = t.named_on_battlefield("Gravecrawler");
    assert_eq!(crawlers.len(), 2);
    let tok = crawlers
        .iter()
        .copied()
        .find(|o| t.obj_now(*o).is_token())
        .unwrap();
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Haste));
    let card_one = crawlers.iter().copied().find(|o| *o != tok).unwrap();
    assert!(!t.obj_now(card_one).has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gravecrawler").len(), 1);
}

#[test]
fn spider_verse_copies_spells_cast_from_elsewhere_once_each_turn() {
    cr!("707.10", "707.10f");
    supported("Spider-Verse");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spider-Verse");
    let s = t.graveyard(P0, "Strike It Rich");
    t.lands(P0, "Mountain", 6);
    t.answer_yes(P0, true);
    t.cast(P0, s)
        .method(mtg_engine::object::CastMethod::Keyword(
            KeywordKind::Flashback,
        ))
        .go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 2);
    // Only once each turn.
    let s2 = t.graveyard(P0, "Strike It Rich");
    t.answer_yes(P0, true);
    t.cast(P0, s2)
        .method(mtg_engine::object::CastMethod::Keyword(
            KeywordKind::Flashback,
        ))
        .go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 3);
}

#[test]
fn lightning_storm_ability_may_choose_a_new_target_for_it() {
    cr!("115.7d", "113.6");
    supported("Lightning Storm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let ls = t.hand(P0, "Lightning Storm");
    let spell = t.cast(P0, ls).target(Entity::Player(P1)).go();
    // P1 discards a land: two more counters and a new target (P0's Bears).
    let land = t.hand(P1, "Forest");
    t.g.turn.priority = Some(P1);
    t.answer(
        P1,
        DecisionKind::Entities,
        mtg_engine::decision::Answer::Entities(vec![obj(land)]),
    );
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[obj(bears)]);
    t.activate(P1, spell, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.g.obj(spell).counter("charge"), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn shiko_and_narset_copies_a_second_spell_that_targets_or_draws() {
    cr!("707.10", "115.9b");
    supported("Shiko and Narset, Unified");
    // The second spell targets a player: it's copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shiko and Narset, Unified");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // The second spell has no targets: draw a card instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shiko and Narset, Unified");
    t.lands(P0, "Island", 3);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    let div = t.hand(P0, "Divination");
    let hand = t.hand_size(P0);
    t.cast(P0, div).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand - 1 + 1 + 2);
}
