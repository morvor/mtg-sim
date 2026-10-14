//! "If that mana is spent on a creature spell, it gains haste." (Hall of the Bandit Lord,
//! Generator Servant, Arena of Glory, Carnelian Orb of Dragonkind, Domri, Chaos Bringer):
//! an additional effect of the mana on the spell it's spent on (CR 106.6), which carries
//! over to the permanent that spell becomes (CR 400.7a).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.g.obj(t.g.current(id)).has_keyword(k)
}

#[test]
fn hall_of_the_bandit_lord_gives_the_creature_haste_for_good() {
    cr!("106.6", "400.7a");
    ruling!(
        "Hall of the Bandit Lord",
        "The effect that grants haste doesn’t wear off at end of turn."
    );
    assert_supported("Hall of the Bandit Lord");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let hall = t.battlefield(P0, "Hall of the Bandit Lord");
    t.lands(P0, "Forest", 1);
    t.activate(P0, hall, 0, &[]).unwrap();
    assert_eq!(t.life(P0), 17);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    // No triggered ability: the mana's effect applies as it's spent.
    t.settle();
    assert_eq!(t.g.stack.len(), 1, "{}", t.dump_log());
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1, "{}", t.dump_log());
    assert!(has(&t, b[0], KeywordKind::Haste), "{}", t.dump_log());
    t.advance_to(P1, Step::PrecombatMain);
    assert!(has(&t, b[0], KeywordKind::Haste));
}

#[test]
fn generator_servant_haste_lasts_until_end_of_turn_and_needs_a_creature_spell() {
    cr!("106.6", "400.7a", "514.2");
    ruling!(
        "Generator Servant",
        "An instant or sorcery spell is not a creature spell, even if that spell creates creature tokens."
    );
    assert_supported("Generator Servant");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let gs = t.battlefield(P0, "Generator Servant");
    t.activate(P0, gs, 0, &[]).unwrap();
    let myr = t.hand(P0, "Alpha Myr");
    t.cast(P0, myr).go();
    t.resolve_all();
    let b = t.named_on_battlefield("Alpha Myr");
    assert_eq!(b.len(), 1, "{}", t.dump_log());
    assert!(has(&t, b[0], KeywordKind::Haste), "{}", t.dump_log());
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!has(&t, b[0], KeywordKind::Haste), "until end of turn");

    // Spent on a noncreature spell: nothing gains haste.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let gs = t.battlefield(P0, "Generator Servant");
    t.activate(P0, gs, 0, &[]).unwrap();
    let sign = t.hand(P0, "Mind Stone");
    t.cast(P0, sign).go();
    t.resolve_all();
    let ms = t.named_on_battlefield("Mind Stone");
    assert_eq!(ms.len(), 1, "{}", t.dump_log());
    assert!(!has(&t, ms[0], KeywordKind::Haste));
}

#[test]
fn carnelian_orb_only_hastes_dragon_creature_spells() {
    cr!("106.6", "400.7a");
    ruling!(
        "Carnelian Orb of Dragonkind",
        "The mana created by Carnelian Orb of Dragonkind can be spent on anything, not just Dragon creature spells."
    );
    assert_supported("Carnelian Orb of Dragonkind");
    for (name, dragon) in [("Shivan Dragon", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let orb = t.battlefield(P0, "Carnelian Orb of Dragonkind");
        t.lands(P0, "Mountain", 5);
        t.activate(P0, orb, 0, &[]).unwrap();
        let c = t.hand(P0, name);
        t.cast(P0, c).go();
        t.resolve_all();
        let p = t.named_on_battlefield(name);
        assert_eq!(p.len(), 1, "{}", t.dump_log());
        assert_eq!(has(&t, p[0], KeywordKind::Haste), dragon, "{name}");
    }
}

#[test]
fn arena_of_glory_mana_spent_on_two_creature_spells_hastes_both() {
    cr!("106.6", "106.6a", "400.7a");
    ruling!(
        "Arena of Glory",
        "If the mana generated with Arena of Glory's last ability is spent on two different creature spells, each of those spells (and resulting creatures) will gain haste until end of turn."
    );
    assert_supported("Arena of Glory");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let arena = t.battlefield(P0, "Arena of Glory");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, arena, 1, &[]).unwrap();
    t.lands(P0, "Forest", 1);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Goblin Piker");
    t.cast(P0, a).go();
    t.resolve_all();
    t.cast(P0, b).go();
    t.resolve_all();
    for name in ["Grizzly Bears", "Goblin Piker"] {
        let p = t.named_on_battlefield(name);
        assert_eq!(p.len(), 1, "{}", t.dump_log());
        assert!(has(&t, p[0], KeywordKind::Haste), "{name}: {}", t.dump_log());
    }
}

#[test]
fn domri_mana_gives_the_creature_riot() {
    cr!("106.6", "400.7a", "702.136a");
    assert_supported("Domri, Chaos Bringer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let domri = t.battlefield(P0, "Domri, Chaos Bringer");
    t.lands(P0, "Forest", 1);
    t.activate(P0, domri, 0, &[]).unwrap();
    // A loyalty ability, not a mana ability: it uses the stack.
    assert_eq!(t.g.stack.len(), 1, "{}", t.dump_log());
    t.resolve_all();
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1, "{}", t.dump_log());
    assert_eq!(t.counters(b[0], "+1/+1"), 1, "{}", t.dump_log());
}
