//! Rulings batch P027 — copies of spells and abilities (CR 707.10): a copy made by a
//! resolving ability uses the spell's last known information, so it's created even if the
//! spell was countered, and it's put on the stack above it (CR 608.2h, 707.10); copies
//! aren't cast (CR 707.10, 601.2i); what a copy copies (additional costs paid, CR 707.10)
//! and what it doesn't (choices made on resolution, CR 707.10); which objects get a copy
//! for "each other creature that spell could target" (CR 115.10, 707.10c).

use crate::r_p027_common::*;
use crate::r_p028_common::{self as p028, Copier};
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::{supported, tokens};
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use crate::r_s17_common::enter_transformed;
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::*;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Lightning Bolt at P1 (3 damage).
fn bolt(t: &mut TestGame) -> ObjectId {
    cast_new(t, P0, "Lightning Bolt", &[Entity::Player(P1)])
}

/// P0 casts Blaze ({X}{R}: "Blaze deals X damage to any target.") at P1 with X = 3.
fn blaze(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let c = t.hand(P0, "Blaze");
    t.cast(P0, c).x(3).target(Entity::Player(P1)).go()
}

/// Unbound Flourishing: "Whenever you cast an instant or sorcery spell or activate an
/// ability, if that spell's mana cost or that ability's activation cost contains {X},
/// copy that spell or ability."
const UNBOUND_FLOURISHING: Copier = Copier {
    name: "Unbound Flourishing",
    setup: |t| {
        t.battlefield(P0, "Unbound Flourishing");
    },
    start: p028::keep,
};

/// `c` copies the spell `cast` makes (dealing 3 damage to P1). If the spell is countered
/// before the copy ability resolves, the copy is still created and deals its damage; if
/// it isn't, the copy is above the spell on the stack (it resolves first).
fn copies_even_if_countered(c: &Copier, cast: fn(&mut TestGame) -> ObjectId) {
    supported(c.name);
    let mut t = TestGame::new(2);
    (c.setup)(&mut t);
    let s = cast(&mut t);
    (c.start)(&mut t, s);
    t.settle();
    assert!(t.stack_len() >= 2, "{}: no copy trigger", c.name);
    assert!(t.g.counter(s, None), "{}", c.name);
    t.g.flush_events();
    t.settle();
    assert!(!t.g.stack.contains(&s));
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1, "{}: no copy of the countered spell", c.name);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", c.name);
    // Not countered: the copy is put on the stack above the original.
    let mut t = TestGame::new(2);
    (c.setup)(&mut t);
    let s = cast(&mut t);
    (c.start)(&mut t, s);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1, "{}", c.name);
    let at = |id| t.g.stack.iter().position(|x| *x == id).unwrap();
    assert!(at(copies[0]) > at(s), "{}", c.name);
    while t.g.stack.contains(&copies[0]) {
        t.resolve();
    }
    // The copy resolved; the original is still waiting.
    assert_eq!(t.life(P1), 17, "{}", c.name);
    assert!(t.g.stack.contains(&s), "{}", c.name);
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "{}", c.name);
}

#[test]
fn copy_triggers_copy_a_countered_spell_and_the_copy_resolves_first() {
    cr!("707.10", "608.2h", "405.5");
    ruling!("Unbound Flourishing", "A copy is created even if the spell or ability that caused Unbound Flourishing's ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell.");
    copies_even_if_countered(&UNBOUND_FLOURISHING, blaze);
    ruling!("Bonus Round", "A copy is created even if the spell that caused Bonus Round’s ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell.");
    copies_even_if_countered(&p028::BONUS_ROUND, bolt);
    ruling!("Double Vision", "A copy is created even if the spell that caused Double Vision's ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell.");
    copies_even_if_countered(&p028::DOUBLE_VISION, bolt);
    ruling!("Doublecast", "A copy is created even if the spell that caused Doublecast’s ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell.");
    copies_even_if_countered(&p028::DOUBLECAST, bolt);
    ruling!("Galvanic Iteration", "A copy is created even if the spell that caused Galvanic Iteration's ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell.");
    copies_even_if_countered(&p028::GALVANIC_ITERATION, bolt);
    ruling!("Kalamax, the Stormsire", "A copy is created even if the spell that caused Kalamax's ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell.");
    copies_even_if_countered(&p028::KALAMAX, bolt);
    ruling!("Jin-Gitaxias, Progress Tyrant", "A copy is created even if the spell that caused the ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell.");
    copies_even_if_countered(&p028::JIN_GITAXIAS, bolt);
}

#[test]
fn unbound_flourishing_copies_a_countered_ability() {
    cr!("707.10", "608.2h");
    ruling!("Unbound Flourishing", "A copy is created even if the spell or ability that caused Unbound Flourishing's ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell.");
    // Cinder Elemental: "{X}{R}, {T}, Sacrifice this creature: It deals X damage to any
    // target." With X = 2, countered: the copy still deals 2.
    supported("Cinder Elemental");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    let ce = t.battlefield(P0, "Cinder Elemental");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, ce, "deals X damage").expect("activation");
    keep_copy_targets(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let ability = t.g.stack[0];
    assert!(t.g.counter(ability, None));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn kalamaxs_counter_trigger_resolves_before_the_copy_even_if_its_countered() {
    cr!("603.3", "603.2", "707.10");
    ruling!("Kalamax, the Stormsire", "An ability that triggers when a player copies a spell resolves before the copy. It resolves even if that spell is countered.");
    // "Whenever you copy an instant spell, put a +1/+1 counter on Kalamax."
    let mut t = TestGame::new(2);
    (p028::KALAMAX.setup)(&mut t);
    let kalamax = t.named_on_battlefield("Kalamax, the Stormsire")[0];
    bolt(&mut t);
    p028::keep(&mut t, kalamax);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    // The counter trigger is above the copy.
    let top = *t.g.stack.last().unwrap();
    assert_ne!(top, copies[0]);
    assert_eq!(abilities_from(&t, kalamax), vec![top]);
    // The copy is countered: the trigger still resolves.
    assert!(t.g.counter(copies[0], None));
    t.g.flush_events();
    t.resolve();
    assert_eq!(t.counters(kalamax, counters::PLUS1), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

/// The number of Elemental tokens P0's Young Pyromancer made ("Whenever you cast an
/// instant or sorcery spell, create a 1/1 red Elemental creature token.").
fn elementals(t: &TestGame) -> usize {
    tokens(t, P0)
        .iter()
        .filter(|x| name_of(t, **x).starts_with("Elemental"))
        .count()
}

#[test]
fn cloven_castings_copy_isnt_cast() {
    cr!("707.10", "601.2i", "603.2");
    ruling!("Cloven Casting", "Abilities that trigger when a spell is cast (such as this ability or cascade) won’t trigger when you create a copy this way.");
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    (p028::CLOVEN_CASTING.setup)(&mut t);
    t.lands(P0, "Wastes", 5);
    t.battlefield(P0, "Young Pyromancer");
    let from = t.asked().len();
    // Lightning Helix (multicolored): Cloven Casting's {1} is paid.
    let helix = cast_new(&mut t, P0, "Lightning Helix", &[Entity::Player(P1)]);
    p028::CLOVEN_CASTING.start.clone()(&mut t, helix);
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "the Helix and its copy");
    // One Elemental (the cast), and Cloven Casting asked once.
    assert_eq!(elementals(&t), 1);
    let asked = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay")))
        .count();
    assert_eq!(asked, 1);
}

#[test]
fn dualcaster_mages_copy_isnt_cast() {
    cr!("707.10", "601.2i", "603.2");
    ruling!("Dualcaster Mage", "As Dualcaster Mage's triggered ability resolves, it creates a copy of a spell on the stack. The copy isn't cast, so abilities that trigger when a player casts a spell won't trigger.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let s = bolt(&mut t);
    (p028::DUALCASTER_MAGE.start)(&mut t, s);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // One Elemental, for Lightning Bolt; none for Dualcaster Mage (a creature) or the copy.
    assert_eq!(elementals(&t), 1);
}

/// P0 starts the delayed copy (`setup`), then casts Divination (no targets): it's copied,
/// so P0 draws four cards.
fn next_spell_copied(setup: fn(&mut TestGame)) {
    let mut t = TestGame::new(2);
    setup(&mut t);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Divination", &[]);
    keep_copy_targets(&mut t, P0);
    assert_eq!(resolve_until_copies(&mut t).len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn the_next_spell_is_copied_whether_or_not_it_has_targets() {
    cr!("603.7a", "707.10");
    ruling!("Adaptive Training Post", "After Adaptive Training Post’s last ability resolves, the next instant or sorcery spell you cast that turn will be copied whether or not it has targets.");
    supported("Adaptive Training Post");
    next_spell_copied(|t| {
        let post = t.battlefield(P0, "Adaptive Training Post");
        t.g.add_counters(obj(post), counters::CHARGE, 3, None);
        activate_containing(t, P0, post, "Remove three").expect("activation");
        t.resolve_all();
        assert_eq!(t.counters(post, counters::CHARGE), 0);
    });
    ruling!("Teach by Example", "After Teach by Example resolves, the next instant or sorcery spell you cast that turn will be copied whether or not it has targets.");
    supported("Teach by Example");
    next_spell_copied(|t| {
        t.lands(P0, "Island", 2);
        let c = t.hand(P0, "Teach by Example");
        t.cast(P0, c).go();
        t.resolve_all();
    });
}

#[test]
fn ether_copies_the_next_spell_even_without_its_mana() {
    cr!("603.7a", "707.10", "106.4");
    ruling!("Ether", "After Ether's ability resolves, the next instant or sorcery spell you cast that turn will be copied whether or not it has targets and whether or not you spent the mana added by Ether's ability on that spell.");
    supported("Ether");
    let mut t = TestGame::new(2);
    let ether = t.battlefield(P0, "Ether");
    assert!(tap_for_mana(&mut t, P0, ether, "Add {U}"));
    // The {U} empties from P0's pool at the end of the main phase.
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(crate::r_s24_common::pool(&t, P0, mtg_engine::mana::ManaType::U), 0);
    // Think Twice (an instant, no targets: "Draw a card."), paid with Islands.
    t.lands(P0, "Island", 2);
    let tt = t.hand(P0, "Think Twice");
    let hand = t.hand_size(P0);
    t.cast(P0, tt).go();
    assert_eq!(resolve_until_copies(&mut t).len(), 1);
    t.resolve_all();
    // Think Twice left the hand; it and its copy drew two cards.
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn primal_wellspring_copies_any_spell_its_mana_is_spent_on() {
    cr!("603.7a", "707.10", "106.6");
    ruling!("Primal Amulet // Primal Wellspring", "Any instant or sorcery spell you spend the mana on will be copied, not just one that requires targets.");
    let mut t = TestGame::new(2);
    let w = enter_transformed(&mut t, P0, "Primal Amulet // Primal Wellspring");
    // Index(1): blue.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert!(tap_for_mana(&mut t, P0, w, "Add one mana"));
    t.lands(P0, "Wastes", 2);
    let d = t.hand(P0, "Divination");
    let hand = t.hand_size(P0);
    t.cast(P0, d).go();
    assert_eq!(resolve_until_copies(&mut t).len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 4);
}

#[test]
fn unbound_flourishing_doesnt_copy_an_x_cost_without_the_x_symbol() {
    cr!("107.3", "602.2b");
    ruling!("Unbound Flourishing", "An ability with a cost that includes X, but not the mana symbol {X}, won't be copied.");
    // Krav, the Unredeemed: "{B}, Sacrifice X creatures: Target player draws X cards and
    // gains X life. Put X +1/+1 counters on Krav."
    supported("Krav, the Unredeemed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    let krav = t.battlefield(P0, "Krav, the Unredeemed");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    activate_containing(&mut t, P0, krav, "Sacrifice X").expect("activation");
    t.settle();
    assert_eq!(t.stack_len(), 1, "it was copied");
    let life = t.life(P0);
    t.resolve_all();
    assert_eq!(t.life(P0), life + 1);
    // Positive control: Cinder Elemental's {X}{R} ability is copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    let ce = t.battlefield(P0, "Cinder Elemental");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, ce, "deals X damage").expect("activation");
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn ricochet_copies_keep_additional_costs_paid_for_the_original() {
    cr!("707.10", "702.33d");
    ruling!("Riku of Two Reflections", "You can't pay any additional costs for the copy. However, effects based on any additional costs paid for the original spell are copied as though those same costs were paid for the copy. Notably, if the original spell was kicked (or kicked a certain number of times), the copy will also be kicked (or kicked that many times).");
    supported("Riku of Two Reflections");
    for kicked in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Riku of Two Reflections");
        t.lands(P0, "Mountain", 5);
        let card = t.hand(P0, "Burst Lightning");
        let from = t.asked().len();
        t.cast(P0, card).kicked(kicked).target(Entity::Player(P1)).go();
        // {U}{R} for Riku's trigger.
        t.lands(P0, "Island", 1);
        t.lands(P0, "Mountain", 1);
        t.answer_yes(P0, true);
        keep_copy_targets(&mut t, P0);
        t.resolve_all();
        let n = if kicked { 4 } else { 2 };
        assert_eq!(t.life(P1), 20 - 2 * n, "kicked: {kicked}");
        // Kicker was offered only for the original.
        let offered = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
            .count();
        assert_eq!(offered, 1);
    }
}

#[test]
fn see_doubles_copy_keeps_the_originals_kicker() {
    cr!("707.10", "702.33d");
    ruling!("See Double", "You can’t choose to pay any alternative or additional costs for the copy of the spell. However, effects based on any alternative or additional costs that were paid for the original spell are copied as though those same costs were paid for the copy.");
    supported("See Double");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Burst Lightning");
    let from = t.asked().len();
    let burst = t.cast(P0, card).kicked(true).target(Entity::Player(P1)).go();
    lands_for_cost(&mut t, P0, "See Double");
    let sd = t.hand(P0, "See Double");
    t.cast(P0, sd).modes(&[0]).target(obj(burst)).go();
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    let offered = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
        .count();
    assert_eq!(offered, 1);
}

#[test]
fn see_double_both_modes_if_any_opponent_has_eight_cards_in_their_graveyard() {
    cr!("700.2", "700.2d");
    ruling!("See Double", "You may choose both modes if any opponent has eight or more cards in their graveyard, not necessarily all of them.");
    for (cards, both) in [(8, true), (7, false)] {
        let mut t = TestGame::new(3);
        for _ in 0..cards {
            t.graveyard(P1, "Grizzly Bears");
        }
        t.set_step(P1, Step::PrecombatMain);
        let s = cast_new(&mut t, P1, "Divination", &[]);
        let bears = t.battlefield(P0, "Hill Giant");
        lands_for_cost(&mut t, P0, "See Double");
        let sd = t.hand(P0, "See Double");
        let before = tokens(&t, P0);
        t.cast(P0, sd)
            .modes(&[0, 1])
            .targets(&[obj(s)])
            .target(obj(bears))
            .go();
        keep_copy_targets(&mut t, P0);
        t.resolve_all();
        let new = tokens(&t, P0).len() - before.len();
        let copied = new == 1;
        assert_eq!(copied, both, "{cards} cards");
    }
}

#[test]
fn sublime_epiphany_counters_a_keyword_trigger() {
    cr!("113.3c", "702.108a", "701.6b");
    ruling!("Sublime Epiphany", "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities (such as prowess) are triggered abilities and will have \"when,\" \"whenever,\" or \"at\" in their reminder text.");
    supported("Sublime Epiphany");
    // P1 casts Shock with a prowess creature (Monastery Swiftspear): P0 counters the
    // prowess trigger with "Counter target activated or triggered ability."
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let swift = t.battlefield(P1, "Monastery Swiftspear");
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.settle();
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(abilities_from(&t, swift), vec![trig]);
    lands_for_cost(&mut t, P0, "Sublime Epiphany");
    let se = t.hand(P0, "Sublime Epiphany");
    t.cast(P0, se).modes(&[1]).target(obj(trig)).go();
    t.resolve();
    assert!(!t.g.stack.contains(&trig), "the prowess trigger wasn't countered");
    t.resolve_all();
    assert_eq!(t.pt(swift), (1, 2));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn sublime_epiphany_chooses_each_mode_at_most_once() {
    cr!("700.2", "700.2d");
    ruling!("Sublime Epiphany", "You can't choose any one mode more than once.");
    // Choosing "Target player draws a card." twice: not a legal choice. P0 draws once.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Sublime Epiphany");
    let se = t.hand(P0, "Sublime Epiphany");
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    t.cast(P0, se)
        .modes(&[4, 4])
        .target(Entity::Player(P0))
        .target(Entity::Player(P0))
        .go();
    let modes = crate::r_s26_common::modes_on_stack(&t, se_on_stack(&t));
    let mut dedup = modes.clone();
    dedup.dedup();
    assert_eq!(modes, dedup, "a mode was chosen twice");
    // The decision offered each mode once and said so.
    let offered = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseModes { modes, .. } => Some(modes.len()),
        _ => None,
    });
    assert_eq!(offered, Some(5));
    t.resolve_all();
    assert!(t.hand_size(P0) <= hand);
}

fn se_on_stack(t: &TestGame) -> ObjectId {
    *t.g.stack.last().unwrap()
}

#[test]
fn lithoform_engine_copies_an_equip_ability() {
    cr!("602.1", "702.6a", "707.10");
    ruling!("Lithoform Engine", "Activated abilities contain a colon. They're generally written “[Cost]: [Effect].” Some keyword abilities (such as equip) are activated abilities and will have a colon in their reminder text.");
    supported("Lithoform Engine");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let sword = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, sword, "Equip").expect("equip");
    let equip = *t.g.stack.last().unwrap();
    t.answer_targets(P0, &[obj(equip)]);
    activate_containing(&mut t, P0, engine, "activated or triggered").expect("copy");
    // The copy attaches it to the Hill Giant, then the original to the Bears.
    change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    t.resolve();
    assert_eq!(spell_copies(&t).len() + t.stack_len(), 2);
    t.resolve();
    assert_eq!(attached_to(&t, sword), Some(obj(giant)));
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(obj(bears)));
}

#[test]
fn lithoform_engines_copy_pays_the_triggers_cost_separately() {
    cr!("707.10", "603.5", "608.2");
    ruling!("Lithoform Engine", "Any choices made when the spell or ability resolves won't have been made yet when it's copied. Any such choices will be made separately when the copy resolves. Most notably, if a triggered ability asks you to pay a cost (such as that of Leyline Tyrant), you pay that cost for the copy if you wish to have it paid.");
    // Bramble Sovereign: "Whenever another nontoken creature enters, you may pay {1}{G}.
    // If you do, that creature's controller creates a token that's a copy of that
    // creature." Copied: P0 pays for the copy, not for the original.
    supported("Bramble Sovereign");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bramble Sovereign");
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.lands(P0, "Wastes", 2);
    t.enter(P0, "Grizzly Bears");
    t.settle();
    let trig = *t.g.stack.last().unwrap();
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(trig)]);
    activate_containing(&mut t, P0, engine, "activated or triggered").expect("copy");
    // Mana for both {1}{G} payments.
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    t.resolve();
    // Nothing paid yet.
    assert_eq!(choices_since(&t, from).len(), 0);
    let paid_before = crate::r_s01_common::tapped_lands(&t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), paid_before + 2);
}

/// P0's Shock (2 damage) cast at `target`, one of P0's 3/3 creatures (that survive it).
fn shock_at(t: &mut TestGame, target: ObjectId) {
    cast_new(t, P0, "Shock", &[obj(target)]);
}

/// Gives the permanent shroud until end of turn.
fn shroud(t: &mut TestGame, id: ObjectId) {
    modify_until_eot(
        t,
        id,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Shroud))],
    );
}

/// After the stack resolves, which of `creatures` were dealt damage.
fn damaged(t: &TestGame, creatures: &[ObjectId]) -> Vec<bool> {
    creatures
        .iter()
        .map(|c| !t.on_battlefield(*c) || t.obj_now(*c).damage > 0)
        .collect()
}

#[test]
fn precursor_golem_ignores_golems_the_spell_couldnt_target() {
    cr!("707.10", "707.10c", "702.18a");
    ruling!("Precursor Golem", "Any Golem that couldn't be targeted by the original spell (due to shroud, protection abilities, targeting restrictions, or any other reason) is just ignored by Precursor Golem's second ability.");
    supported("Precursor Golem");
    let mut t = TestGame::new(2);
    let pg = t.enter(P0, "Precursor Golem");
    t.resolve_all();
    let golems = crate::r_s01_common::with_subtype(&t, P0, "Golem");
    assert_eq!(golems.len(), 3);
    let others: Vec<ObjectId> = golems.iter().copied().filter(|g| *g != pg).collect();
    shroud(&mut t, others[1]);
    shock_at(&mut t, pg);
    t.resolve_all();
    assert_eq!(damaged(&t, &[pg, others[0], others[1]]), vec![true, true, false]);
}

#[test]
fn zada_ignores_creatures_the_spell_couldnt_target() {
    cr!("707.10", "707.10c", "702.18a");
    ruling!("Zada, Hedron Grinder", "Any creature you control that couldn't be targeted by the original spell (due to shroud, protection abilities, targeting restrictions, or any other reason) is just ignored by Zada's ability.");
    supported("Zada, Hedron Grinder");
    let mut t = TestGame::new(2);
    let zada = t.battlefield(P0, "Zada, Hedron Grinder");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Hill Giant");
    let p1 = t.battlefield(P1, "Hill Giant");
    shroud(&mut t, b);
    shock_at(&mut t, zada);
    t.resolve_all();
    assert_eq!(damaged(&t, &[zada, a, b, p1]), vec![true, true, false, false]);
}

#[test]
fn mirrorwing_dragon_ignores_creatures_the_spell_couldnt_target() {
    cr!("707.10", "707.10c", "702.18a");
    supported("Mirrorwing Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Mirrorwing Dragon");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Hill Giant");
    shroud(&mut t, b);
    shock_at(&mut t, dragon);
    t.resolve_all();
    assert_eq!(damaged(&t, &[dragon, a, b]), vec![true, true, false]);
}
