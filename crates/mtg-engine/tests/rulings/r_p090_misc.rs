//! Rulings batch P090 — miscellany: effects on a set of creatures fixed as they resolve
//! (Invert the Skies, Unnerving Assault, Chittering Host; CR 611.2c), mana value on the
//! battlefield (Ashling's Prerogative; CR 202.3e), haste anthems (CR 302.6, 702.10),
//! flash (King Cheetah, Ghitu Fire; CR 702.8, 601.3, 202.3), Phyrexian Censor's spell
//! limit (CR 604.1) and Psychic Symbiont.

use crate::r_s01_common::*;
use crate::r_s02_common::can_attack;
use crate::r_s06_common::{give_control, has_kw};
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::castable;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// P0 casts `spell` ({3}{G/U} or {2}{U/R}) paying with a land of each color and Wastes.
fn cast_both_colors(t: &mut TestGame, spell: &str, colors: [&str; 2], wastes: usize) {
    t.lands(P0, colors[0], 1);
    t.lands(P0, colors[1], 1);
    t.lands(P0, "Wastes", wastes);
    let s = t.hand(P0, spell);
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(tapped_lands(t, P0), 2 + wastes, "{spell}");
}

#[test]
fn invert_the_skies_and_unnerving_assault_affect_only_creatures_there_as_they_resolve() {
    cr!("611.2c", "106.1", "613.1f");
    ruling!(
        "Invert the Skies",
        "Invert the Skies affects only creatures on the battlefield at the time it resolves. If a creature enters later that turn, it won't be affected."
    );
    ruling!(
        "Invert the Skies",
        "If a creature changes controllers later in the turn, that won't change how Invert the Skies affected it."
    );
    ruling!(
        "Unnerving Assault",
        "Unnerving Assault affects only creatures on the battlefield at the time it resolves. If a creature enters later that turn, it won't be affected."
    );
    ruling!(
        "Unnerving Assault",
        "If a creature changes controllers later in the turn, that won't change how Unnerving Assault affected it."
    );
    supported("Invert the Skies");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Wind Drake");
    cast_both_colors(&mut t, "Invert the Skies", ["Forest", "Island"], 2);
    assert!(has_kw(&t, mine, KeywordKind::Flying));
    assert!(!has_kw(&t, theirs, KeywordKind::Flying));
    // Control swaps: the Bears keep flying under P1, the Drake doesn't regain it.
    give_control(&mut t, mine, P1);
    give_control(&mut t, theirs, P0);
    assert!(has_kw(&t, mine, KeywordKind::Flying));
    assert!(!has_kw(&t, theirs, KeywordKind::Flying));
    // Later arrivals are unaffected.
    let later_mine = t.battlefield(P0, "Hill Giant");
    let later_theirs = t.battlefield(P1, "Wind Drake");
    t.g.recompute();
    assert!(!has_kw(&t, later_mine, KeywordKind::Flying));
    assert!(has_kw(&t, later_theirs, KeywordKind::Flying));

    supported("Unnerving Assault");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    cast_both_colors(&mut t, "Unnerving Assault", ["Island", "Mountain"], 1);
    assert_eq!(t.pt(mine), (3, 2));
    assert_eq!(t.pt(theirs), (2, 3));
    give_control(&mut t, mine, P1);
    give_control(&mut t, theirs, P0);
    assert_eq!(t.pt(mine), (3, 2));
    assert_eq!(t.pt(theirs), (2, 3));
    let later_mine = t.battlefield(P0, "Grizzly Bears");
    let later_theirs = t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(later_mine), (2, 2));
    assert_eq!(t.pt(later_theirs), (3, 3));
}

#[test]
fn chittering_host_affects_only_creatures_there_as_its_ability_resolves() {
    cr!("611.2c", "712.4a");
    // The ruling describes Chittering Host, the meld result of Graf Rats.
    ruling!(
        "Graf Rats",
        "The set of creatures affected by Chittering Host’s triggered ability is determined as the ability resolves. Creatures you begin to control later in the turn won’t get +1/+0 or gain menace."
    );
    supported("Graf Rats");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Graf Rats");
    t.battlefield(P0, "Midnight Scavengers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Chittering Host").len(), 1);
    assert_eq!(t.pt(bears), (3, 2));
    assert!(has_kw(&t, bears, KeywordKind::Menace));
    // A creature P0 begins to control afterwards.
    let theirs = t.battlefield(P1, "Hill Giant");
    give_control(&mut t, theirs, P0);
    let later = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(theirs), (3, 3));
    assert_eq!(t.pt(later), (2, 2));
    assert!(!has_kw(&t, later, KeywordKind::Menace));
}

#[test]
fn ashlings_prerogative_x_is_zero_on_the_battlefield() {
    cr!("202.3e", "107.3", "614.1c");
    ruling!(
        "Ashling's Prerogative",
        "If a creature has X in its mana cost, that X is treated as 0 for the purposes of these effects. It doesn't matter what the value of X was while the creature was on the stack."
    );
    supported("Ashling's Prerogative");
    supported("Endless One");
    // Odd chosen: Endless One cast with X = 3 (mana value 3 on the stack) has mana value 0
    // as it enters, so it enters tapped and lacks haste.
    for (i, odd) in [(0usize, true), (1, false)] {
        let mut t = TestGame::new(2);
        t.answer(
            P0,
            DecisionKind::Option,
            mtg_engine::decision::Answer::Index(i),
        );
        t.enter(P0, "Ashling's Prerogative");
        t.lands(P0, "Wastes", 3);
        let e = t.hand(P0, "Endless One");
        t.cast(P0, e).x(3).go();
        t.resolve_all();
        let one = t.g.current(e);
        assert!(t.on_battlefield(one));
        assert_eq!(t.counters(one, "+1/+1"), 3);
        assert_eq!(t.obj_now(one).tapped, odd, "odd chosen: {odd}");
        assert_eq!(has_kw(&t, one, KeywordKind::Haste), !odd);
    }
}

#[test]
fn haste_anthems_give_haste_to_themselves_and_everyone() {
    cr!("302.6", "702.10b", "611.3a");
    ruling!(
        "Cyclops of Eternal Fury",
        "Cyclops of Eternal Fury's ability gives itself haste."
    );
    ruling!("Madrush Cyclops", "Madrush Cyclops gives itself haste.");
    ruling!(
        "Concordant Crossroads",
        "This ability affects all creatures, not just the ones you control."
    );
    for name in ["Cyclops of Eternal Fury", "Madrush Cyclops"] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield_sick(P0, name);
        to_beginning_of_combat(&mut t, P0);
        assert!(can_attack(&mut t, c), "{name}");
    }
    supported("Concordant Crossroads");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Concordant Crossroads");
    let theirs = t.battlefield_sick(P1, "Grizzly Bears");
    t.g.recompute();
    assert!(has_kw(&t, theirs, KeywordKind::Haste));
    to_beginning_of_combat(&mut t, P1);
    assert!(can_attack(&mut t, theirs));
}

#[test]
fn king_cheetah_is_always_a_creature_spell() {
    cr!("702.8a", "304.1", "302.1");
    ruling!(
        "King Cheetah",
        "King Cheetah is always a Creature spell, never an instant, regardless of when you cast it."
    );
    supported("King Cheetah");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    give_mana_for(&mut t, P0, "King Cheetah");
    let k = t.hand(P0, "King Cheetah");
    let s = t.cast(P0, k).go();
    let o = t.obj_now(s);
    assert_eq!(o.zone, Zone::Stack);
    assert!(o.chars.is(CardType::Creature));
    assert!(!o.chars.is(CardType::Instant));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("King Cheetah").len(), 1);
}

#[test]
fn ghitu_fire_flash_cost_isnt_part_of_its_mana_cost() {
    cr!("202.3", "601.2f", "118.8d");
    ruling!(
        "Ghitu Fire",
        "The extra 2 mana to cast it as an instant does not count as part of the mana cost."
    );
    supported("Ghitu Fire");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    t.lands(P0, "Mountain", 5);
    let gf = t.hand(P0, "Ghitu Fire");
    let uid = t.obj(gf).chars.abilities[0].uid;
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P0);
    let s =
        t.g.cast_spell(P0, gf, CastMethod::Alternative(uid))
            .expect("cast with flash");
    assert_eq!(tapped_lands(&t, P0), 5, "{{2}}{{R}} and {{2}} more");
    assert_eq!(crate::r_s08_common::mana_value(&t, s), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn phyrexian_censor_counts_non_phyrexian_spells() {
    cr!("604.1", "205.3m", "101.2");
    ruling!(
        "Phyrexian Censor",
        "A “non-Phyrexian spell” is any spell that doesn’t have the Phyrexian creature type, even if “Phyrexian” appears in its name. Artifact spells, battle spells, and Goblin creature spells are examples of non-Phyrexian spells."
    );
    ruling!(
        "Phyrexian Censor",
        "If a player casts a non-Phyrexian spell that was countered, they can’t cast another non-Phyrexian spell during the same turn."
    );
    supported("Phyrexian Censor");
    supported("Phyrexian Rager");
    // Phyrexian Arena (an enchantment named "Phyrexian") is a non-Phyrexian spell.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Phyrexian Censor");
    t.lands(P0, "Swamp", 6);
    let arena = t.hand(P0, "Phyrexian Arena");
    t.cast(P0, arena).go();
    t.resolve_all();
    let thopter = t.hand(P0, "Ornithopter");
    assert!(
        !castable(&mut t, P0, thopter),
        "an artifact spell is non-Phyrexian"
    );
    // A Phyrexian creature spell is still allowed.
    let rager = t.hand(P0, "Phyrexian Rager");
    assert!(castable(&mut t, P0, rager));
    // A countered non-Phyrexian spell counts.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Phyrexian Censor");
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let s = t.cast(P0, bolt).target(P1).go();
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(s).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P1), 20);
    let shock = t.hand(P0, "Shock");
    assert!(!castable(&mut t, P0, shock));
}

#[test]
fn psychic_symbiont_draws_even_if_the_opponent_has_no_cards() {
    cr!("115.1", "609.3");
    ruling!(
        "Psychic Symbiont",
        "Psychic Symbiont's last ability can target an opponent with no cards in hand. You'll draw a card even if no opponent discards a card."
    );
    supported("Psychic Symbiont");
    let mut t = TestGame::new(2);
    assert_eq!(t.hand_size(P1), 0);
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Psychic Symbiont");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
