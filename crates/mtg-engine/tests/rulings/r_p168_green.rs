//! Rulings batch P168, green synergies: Kruphix's colorless mana keeping its riders (CR
//! 106.6), Momir Vig's two cast triggers (CR 603.3b), Garruk's emblem, Green Sun's Zenith's
//! shuffles (CR 701.24), devotion counted on resolution (CR 700.5, 608.2h), costs paid
//! while no one can respond (Natural Order, Magus of the Order: CR 601.2h, 602.2b),
//! Stampeding Wildebeests' untargeted return, Nettle Sentinel's trigger resolving before
//! its spell, Hunter's Ambush's prevention (CR 615), and Xenagos's type change working only
//! on the battlefield.

use crate::r_s01_common::*;
use crate::r_s03_common::{priority_asked_since, to_blockers};
use crate::r_s06_common::give_control;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
        .count()
}

/// Casts the real card `name` from `p`'s hand, giving `p` the lands for its mana cost.
fn cast_new(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, c).go()
}

#[test]
fn kruphix_colorless_mana_keeps_its_spending_restriction() {
    cr!("106.6", "106.4");
    ruling!(
        "Kruphix, God of Horizons",
        "If unspent mana has any restrictions or riders associated with it (for example, if it was produced by Cavern of Souls), those restrictions or riders will remain associated with that mana when it becomes colorless."
    );
    supported("Kruphix, God of Horizons");
    supported("Delighted Halfling");
    fn restricted_colorless() -> TestGame {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Kruphix, God of Horizons");
        let h = t.battlefield(P0, "Delighted Halfling");
        // Delighted Halfling's green mana: only for legendary spells.
        t.answer(P0, DecisionKind::Option, Answer::Index(4));
        t.activate(P0, h, 1, &[]).unwrap();
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 1);
        // The main phase ends: the mana becomes colorless instead of emptying.
        t.advance_to(P0, Step::PostcombatMain);
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 0);
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
        t
    }
    // It still can't pay for a nonlegendary spell ...
    let mut t = restricted_colorless();
    let sol = t.hand(P0, "Sol Ring");
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, sol).try_go().is_err());
    // ... but pays for a legendary one, which can't be countered.
    let mut t = restricted_colorless();
    let spear = t.hand(P0, "Shadowspear");
    let spell = t.cast(P0, spear).go();
    t.g.recompute();
    assert!(t
        .g
        .obj(spell)
        .chars
        .abilities
        .iter()
        .any(|a| a.text == "This spell can't be countered."));
}

#[test]
fn momir_vig_both_triggers_stack_in_either_order() {
    cr!("603.3b", "701.23a");
    ruling!(
        "Momir Vig, Simic Visionary",
        "If you cast a green and blue creature spell, both abilities will trigger. The abilities may be stacked so that when they resolve, you search your library for a creature card, put it on top of your library, and then put that card into your hand. The abilities may also be stacked in the other order, of course."
    );
    supported("Momir Vig, Simic Visionary");
    let mut outcomes = vec![];
    for order in [[0usize, 1], [1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Momir Vig, Simic Visionary");
        let dragon = t.library_top(P0, "Shivan Dragon");
        t.library_top(P0, "Island");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(dragon)]);
        cast_new(&mut t, P0, "Coiling Oracle", &[]);
        t.settle();
        // Both abilities triggered.
        assert_eq!(t.stack_len(), 3);
        // Resolve the two triggers (not the spell).
        t.resolve();
        t.resolve();
        let d = t.g.current(dragon);
        outcomes.push((t.zone(d), t.g.player(P0).library.last() == Some(&d)));
    }
    outcomes.sort_by_key(|o| format!("{o:?}"));
    // One order: the search puts it on top, then the reveal puts it into the hand. The
    // other: the reveal finds the Island, then the search puts the Dragon on top.
    assert!(outcomes.contains(&(Zone::Hand(P0), false)), "{outcomes:?}");
    assert!(outcomes.contains(&(Zone::Library(P0), true)), "{outcomes:?}");
}

#[test]
fn garruk_emblem_creature_enters_first_and_does_not_retrigger() {
    cr!("603.2", "601.2i");
    ruling!(
        "Garruk, Caller of Beasts",
        "If you have Garruk's emblem, the creature card you search for will be put onto the battlefield before the creature spell you cast resolves. The creature card you put onto the battlefield isn't cast, so the ability of the emblem won't retrigger."
    );
    supported("Garruk, Caller of Beasts");
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk, Caller of Beasts");
    let have = t.counters(garruk, "loyalty");
    t.g.add_counters(obj(garruk), "loyalty", 7 - have, None);
    t.activate(P0, garruk, 2, &[]).unwrap();
    t.resolve_all();
    let dragon = t.library_top(P0, "Shivan Dragon");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(dragon)]);
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The Dragon is on the battlefield; the Bears are still on the stack.
    assert_eq!(t.named_on_battlefield("Shivan Dragon").len(), 1);
    assert!(t.g.stack.contains(&bears));
    // The Dragon wasn't cast: nothing new triggered.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn garruk_minus_three_chooses_the_card_on_resolution() {
    cr!("608.2", "606.3");
    ruling!(
        "Garruk, Caller of Beasts",
        "You choose what green creature card to put onto the battlefield, if any, when the second ability resolves."
    );
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk, Caller of Beasts");
    t.activate(P0, garruk, 1, &[]).unwrap();
    // A green creature card that reaches the hand after the activation can be chosen.
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Or none.
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk, Caller of Beasts");
    t.hand(P0, "Grizzly Bears");
    t.activate(P0, garruk, 1, &[]).unwrap();
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn green_suns_zenith_shuffles_its_owners_library_twice() {
    cr!("701.24a", "701.23a");
    ruling!(
        "Green Sun's Zenith",
        "In most cases, if you own Green Sun's Zenith and cast it, you'll shuffle your library twice. In practice, shuffling once is sufficient, but effects that care about you shuffling your library (like Psychogenic Probe, for example) will see that you've shuffled twice."
    );
    supported("Psychogenic Probe");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Psychogenic Probe");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.lands(P0, "Forest", 3);
    let gsz = t.hand(P0, "Green Sun's Zenith");
    t.cast(P0, gsz).x(2).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.zone(t.g.current(gsz)), Zone::Library(P0));
    assert_eq!(shuffles(&t, P0), 2);
    // Psychogenic Probe saw both.
    assert_eq!(t.life(P0), 16);
}

#[test]
fn ivy_lane_denizen_can_target_itself_or_the_entering_creature() {
    cr!("603.3d", "115.1");
    ruling!(
        "Ivy Lane Denizen",
        "Ivy Lane Denizen can be chosen as the target of its own ability."
    );
    ruling!(
        "Ivy Lane Denizen",
        "The green creature that entered the battlefield can be chosen as the target of Ivy Lane Denizen's ability."
    );
    supported("Ivy Lane Denizen");
    for itself in [true, false] {
        let mut t = TestGame::new(2);
        let denizen = t.battlefield(P0, "Ivy Lane Denizen");
        if itself {
            t.answer_targets(P0, &[obj(denizen)]);
        }
        let bears = t.enter(P0, "Grizzly Bears");
        if !itself {
            t.answer_targets(P0, &[obj(bears)]);
        }
        t.resolve_all();
        let (on, off) = if itself { (denizen, bears) } else { (bears, denizen) };
        assert_eq!(t.counters(on, "+1/+1"), 1, "itself: {itself}");
        assert_eq!(t.counters(off, "+1/+1"), 0, "itself: {itself}");
    }
}

#[test]
fn klothyss_design_fixes_x_and_the_affected_set_on_resolution() {
    cr!("611.2c", "700.5", "608.2h");
    ruling!(
        "Klothys's Design",
        "Klothys’s Design affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +X/+X."
    );
    ruling!(
        "Klothys's Design",
        "The value of X is determined only as Klothys’s Design resolves. Once that happens, the value of X won’t change later in the turn even if your devotion to green changes."
    );
    supported("Klothys's Design");
    let mut t = TestGame::new(2);
    // Devotion to green 2 (Kalonian Tusker's {G}{G}).
    let tusker = t.battlefield(P0, "Kalonian Tusker");
    cast_new(&mut t, P0, "Klothys's Design", &[]);
    t.resolve_all();
    assert_eq!(t.pt(tusker), (5, 5));
    // A creature that arrives later doesn't get the bonus; the devotion rose to 3, but the
    // Tusker's bonus is still +2/+2.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(elves), (1, 1));
    assert_eq!(t.pt(tusker), (5, 5));
}

#[test]
fn nyleas_huntmaster_fixes_x_on_resolution() {
    cr!("611.2c", "700.5");
    ruling!(
        "Nylea's Huntmaster",
        "The value of X is determined only as the triggered ability resolves. Once that happens, the value of X won’t change later in the turn even if your devotion to green changes."
    );
    supported("Nylea's Huntmaster");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Nylea's Huntmaster");
    t.resolve_all();
    // Devotion to green: Huntmaster's {G} and the Bears' {G}.
    assert_eq!(t.pt(bears), (4, 2));
    t.battlefield(P0, "Kalonian Tusker");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn march_of_burgeoning_life_targets_less_than_x_and_has_mana_value_x_plus_one() {
    cr!("601.2c", "202.3e", "601.2f");
    ruling!(
        "March of Burgeoning Life",
        "March of Burgeoning Life's target must have a mana value less than X. A creature with a mana value equal to X is not a legal target."
    );
    ruling!(
        "March of Burgeoning Life",
        "The mana value of March of Burgeoning Life while it's on the stack is the value chosen for X plus 1, no matter how much mana you actually paid to cast it."
    );
    supported("March of Burgeoning Life");
    // Grizzly Bears has mana value 2: X = 2 can't target it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let march = t.hand(P0, "March of Burgeoning Life");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    assert!(t.cast(P0, march).x(2).target(bears).try_go().is_err());
    // X = 3 can, and exiling a green card pays {2} of it: mana value 4 all the same.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let elves = t.hand(P0, "Llanowar Elves");
    let march = t.hand(P0, "March of Burgeoning Life");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[obj(elves)]);
    let spell = t.cast(P0, march).x(3).target(bears).go();
    assert_eq!(t.zone(t.g.current(elves)), Zone::Exile);
    assert_eq!(t.g.mana_value_of(spell), 4);
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn march_of_burgeoning_life_cant_cost_less_than_g() {
    cr!("601.2f", "118.7");
    ruling!(
        "March of Burgeoning Life",
        "You may exile more cards than necessary for March of Burgeoning Life's first ability, but you can't reduce the mana it costs to less than {G} this way."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let a = t.hand(P0, "Llanowar Elves");
    let b = t.hand(P0, "Elvish Mystic");
    let march = t.hand(P0, "March of Burgeoning Life");
    // X = 3 is {3}{G}; exiling two green cards reduces it by {4}: it still costs {G}.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    let r = t.cast(P0, march).x(3).target(bears).try_go();
    for (p, d) in t.asked() { if !matches!(d, Decision::Priority{..}) { eprintln!("{p:?} {d:?}"); } }
    r.unwrap();
    assert_eq!(t.zone(t.g.current(a)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(b)), Zone::Exile);
    assert_eq!(tapped_lands(&t, P0), 1);
    // With no mana at all, it can't be cast that way.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.hand(P0, "Llanowar Elves");
    let b = t.hand(P0, "Elvish Mystic");
    let march = t.hand(P0, "March of Burgeoning Life");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    assert!(t.cast(P0, march).x(3).target(bears).try_go().is_err());
}

/// P0 casts a green spell (Giant Growth on Grizzly Bears) and P1 counters it with
/// Counterspell; returns the game after the counterspell resolved, with the triggers the
/// green spell caused still on the stack.
fn cast_green_then_counter(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = cast_new(t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    // The trigger is above the spell.
    assert!(t.stack_len() >= 2);
    assert_eq!(t.g.stack[0], spell);
    cast_new(t, P1, "Counterspell", &[obj(spell)]);
    t.resolve();
    assert_eq!(t.zone(t.g.current(spell)), Zone::Graveyard(P0));
    spell
}

#[test]
fn nettle_sentinel_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Nettle Sentinel",
        "Nettle Sentinel's triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    let sentinel = t.battlefield(P0, "Nettle Sentinel");
    t.g.tap(sentinel);
    t.answer_yes(P0, true);
    cast_green_then_counter(&mut t);
    t.resolve_all();
    assert!(!t.obj_now(sentinel).tapped);
}

#[test]
fn natural_order_and_magus_costs_are_paid_without_anyone_responding() {
    cr!("601.2h", "602.2b", "117.1");
    ruling!(
        "Natural Order",
        "Players can respond to this spell only after it's been cast and all its costs have been paid. No one can try to destroy the creature you sacrificed to stop you from casting this spell or to make you sacrifice a different one."
    );
    ruling!(
        "Magus of the Order",
        "Once you declare you're activating the ability, players can't take any actions until you've finished doing so. Notably, opponents can't try to remove Magus of the Order or green creatures you control to stop you from activating the ability."
    );
    supported("Magus of the Order");
    // Natural Order: the Bears are sacrificed as it's cast; no player gets priority in
    // between; once cast, removing creatures doesn't stop it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dragon = t.library_top(P0, "Craw Wurm");
    give_mana_for(&mut t, P0, "Natural Order");
    let order = t.hand(P0, "Natural Order");
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.cast(P0, order).go();
    assert!(priority_asked_since(&t, from).is_empty());
    assert_eq!(t.zone(t.g.current(bears)), Zone::Graveyard(P0));
    t.answer_choose(P0, &[obj(dragon)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    // Magus: both creatures are sacrificed during the activation.
    let mut t = TestGame::new(2);
    let magus = t.battlefield(P0, "Magus of the Order");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.library_top(P0, "Craw Wurm");
    t.lands(P0, "Forest", 1);
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.activate(P0, magus, 0, &[]).unwrap();
    assert!(priority_asked_since(&t, from).is_empty());
    assert_eq!(t.zone(t.g.current(magus)), Zone::Graveyard(P0));
    assert_eq!(t.zone(t.g.current(bears)), Zone::Graveyard(P0));
    t.answer_choose(P0, &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn natural_order_and_magus_sacrifice_exactly_one_green_creature() {
    cr!("601.2b", "601.2h", "118.3");
    ruling!(
        "Natural Order",
        "Sacrificing a green creature is part of Natural Order's cost. You can't sacrifice more creatures to search for more creature cards, and you can't cast Natural Order at all if you control no green creatures."
    );
    ruling!(
        "Magus of the Order",
        "You can't sacrifice additional creatures to search for more green creatures."
    );
    // No green creature: Natural Order can't be cast (a nongreen one doesn't do).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging Goblin");
    give_mana_for(&mut t, P0, "Natural Order");
    let order = t.hand(P0, "Natural Order");
    assert!(t.cast(P0, order).try_go().is_err());
    // Two green creatures: the cost asks for exactly one; one card is found.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Kalonian Tusker");
    give_mana_for(&mut t, P0, "Natural Order");
    let order = t.hand(P0, "Natural Order");
    t.answer_choose(P0, &[obj(a)]);
    let from = t.asked().len();
    t.cast(P0, order).go();
    assert_eq!(sacrifice_choice_max(&t, from), vec![1]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
    let found = t.named_on_battlefield("Craw Wurm").len()
        + t.named_on_battlefield("Kalonian Tusker").len();
    assert_eq!(found, 1);
    // Magus: with two other green creatures, it sacrifices itself and exactly one of them.
    let mut t = TestGame::new(2);
    let magus = t.battlefield(P0, "Magus of the Order");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    t.library_top(P0, "Craw Wurm");
    t.lands(P0, "Forest", 1);
    t.answer_choose(P0, &[obj(a)]);
    let from = t.asked().len();
    t.activate(P0, magus, 0, &[]).unwrap();
    assert_eq!(sacrifice_choice_max(&t, from), vec![1]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

/// The `max` of the entity choices asked since `from` (a sacrifice cost's choice).
fn sacrifice_choice_max(t: &TestGame, from: usize) -> Vec<u32> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { max, .. } => Some(*max),
            _ => None,
        })
        .collect()
}

#[test]
fn patron_of_the_orochi_untaps_nonbasic_forests_and_multicolored_green_creatures() {
    cr!("205.3i", "105.2");
    ruling!(
        "Patron of the Orochi",
        "Patron of the Orochi’s ability untaps nonbasic lands that are also Forests (such as some of the Revised dual lands) and multicolored creatures that are also green."
    );
    supported("Patron of the Orochi");
    let mut t = TestGame::new(2);
    let patron = t.battlefield(P0, "Patron of the Orochi");
    let taiga = t.battlefield(P0, "Taiga");
    let oracle = t.battlefield(P0, "Coiling Oracle");
    let goblin = t.battlefield(P0, "Raging Goblin");
    for id in [taiga, oracle, goblin] {
        t.g.tap(id);
    }
    t.activate(P0, patron, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(taiga).tapped);
    assert!(!t.obj_now(oracle).tapped);
    assert!(t.obj_now(goblin).tapped);
}

#[test]
fn patron_of_the_orochi_once_each_turn_even_after_a_control_change() {
    cr!("602.5b");
    ruling!(
        "Patron of the Orochi",
        "The Patron’s ability can be activated only once per turn, even if control of Patron of the Orochi changes."
    );
    let mut t = TestGame::new(2);
    let patron = t.battlefield(P0, "Patron of the Orochi");
    t.activate(P0, patron, 0, &[]).unwrap();
    t.resolve_all();
    // The Patron untapped itself (it's green): P0 can't activate it again.
    assert!(!t.obj_now(patron).tapped);
    assert!(t.activate(P0, patron, 0, &[]).is_err());
    // Neither can P1 after gaining control of it this turn (with haste from Fervor, so
    // only the once-each-turn restriction stops it).
    t.battlefield(P1, "Fervor");
    give_control(&mut t, patron, P1);
    assert_eq!(t.obj_now(patron).controller, P1);
    assert!(t.activate(P1, patron, 0, &[]).is_err());
    // Had it not been activated this turn, P1 could.
    let mut t = TestGame::new(2);
    let patron = t.battlefield(P0, "Patron of the Orochi");
    t.battlefield(P1, "Fervor");
    give_control(&mut t, patron, P1);
    assert!(t.activate(P1, patron, 0, &[]).is_ok());
}

#[test]
fn stampeding_wildebeests_returns_an_untargetable_creature_or_itself() {
    cr!("603.3d", "702.16b", "608.2");
    ruling!(
        "Stampeding Wildebeests",
        "The ability that returns a creature doesn’t target anything. You can return a creature with shroud or protection from green, for example. You don’t decide which creature to return until the ability resolves. If you control no other green creatures, you must return Stampeding Wildebeests itself."
    );
    // A green creature with protection from green can be returned.
    let mut t = TestGame::new(2);
    let beasts = t.battlefield(P0, "Stampeding Wildebeests");
    let mut def = custom_card(
        "Green Ward Elf",
        "Creature — Elf",
        "{G}",
        Some((1, 1)),
        "Protection from green",
    );
    def.faces[0].chars.colors = ColorSet::single(Color::Green);
    let pro = t.custom(P0, def, Zone::Battlefield);
    let wolf = t.battlefield(P0, "Sacred Wolf");
    t.answer_choose(P0, &[obj(pro)]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(pro)), Zone::Hand(P0));
    assert!(t.on_battlefield(beasts));
    // And the shroud creature on the next upkeep, which was a choice on resolution among
    // all three.
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(wolf)]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(wolf)), Zone::Hand(P0));
    assert!(t.on_battlefield(beasts));
    assert!(asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.contains(&obj(beasts)))));
    // Alone, it returns itself.
    let mut t = TestGame::new(2);
    let beasts = t.battlefield(P0, "Stampeding Wildebeests");
    t.battlefield(P0, "Raging Goblin");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(beasts)), Zone::Hand(P0));
}

#[test]
fn two_stampeding_wildebeests_return_two_creatures() {
    cr!("603.3b", "608.2");
    ruling!(
        "Stampeding Wildebeests",
        "You choose a creature and return it all during resolution of the triggered ability. This means that if you have two Wildebeests, there is no way to end up returning only one creature."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Stampeding Wildebeests");
    let b = t.battlefield(P0, "Stampeding Wildebeests");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(a)), Zone::Hand(P0));
    assert_eq!(t.zone(t.g.current(b)), Zone::Hand(P0));
}

#[test]
fn wurms_tooth_triggers_on_any_players_green_spell() {
    cr!("603.2");
    ruling!(
        "Wurm's Tooth",
        "The ability triggers whenever any player, not just you, casts a green spell."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wurm's Tooth");
    t.answer_yes(P0, true);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Grizzly Bears", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn karametras_acolyte_is_a_mana_ability() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Karametra's Acolyte",
        "The activated ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    let mut t = TestGame::new(2);
    let acolyte = t.battlefield(P0, "Karametra's Acolyte");
    let from = t.asked().len();
    t.activate(P0, acolyte, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert!(priority_asked_since(&t, from).is_empty());
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 1);
}

#[test]
fn baru_triggers_on_any_forest_but_pumps_only_your_creatures() {
    cr!("603.2", "611.2c");
    ruling!(
        "Baru, Fist of Krosa",
        "The first ability triggers when a Forest enters under any player's control, but only your creatures get the bonus."
    );
    supported("Baru, Fist of Krosa");
    let mut t = TestGame::new(2);
    let baru = t.battlefield(P0, "Baru, Fist of Krosa");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let forest = t.hand(P1, "Forest");
    t.play_land(P1, forest).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(baru), (5, 5));
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn barus_wurm_size_is_fixed_on_resolution() {
    cr!("608.2h", "702.103a");
    ruling!(
        "Baru, Fist of Krosa",
        "The size of the Wurm token is determined when the activated ability resolves. It won't change as the number of lands you control changes."
    );
    let mut t = TestGame::new(2);
    let baru = t.battlefield(P0, "Baru, Fist of Krosa");
    t.lands(P0, "Forest", 3);
    let other = t.hand(P0, "Baru, Fist of Krosa");
    t.answer_choose(P0, &[obj(other)]);
    t.activate(P0, baru, 0, &[]).unwrap();
    t.resolve_all();
    let wurm = with_subtype(&t, P0, "Wurm");
    assert_eq!(wurm.len(), 1);
    assert_eq!(t.pt(wurm[0]), (3, 3));
    t.lands(P0, "Plains", 2);
    t.g.recompute();
    assert_eq!(t.pt(wurm[0]), (3, 3));
}

#[test]
fn hunters_ambush_prevents_damage_from_creatures_that_are_nongreen_when_dealing_it() {
    cr!("615.1", "611.2c", "510.2");
    ruling!(
        "Hunter's Ambush",
        "The prevention effect will apply to any combat damage that would be dealt this turn by a nongreen creature, even if that creature was green or wasn’t on the battlefield when Hunter’s Ambush resolved."
    );
    supported("Hunter's Ambush");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P1, "Hunter's Ambush", &[]);
    t.resolve_all();
    // A nongreen creature that arrived later, and the Bears turned blue.
    let goblin = t.battlefield(P0, "Raging Goblin");
    cast_new(&mut t, P0, "Cerulean Wisps", &[obj(bears)]);
    t.resolve_all();
    t.attack(&[(goblin, pl(P1)), (bears, pl(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    // A green creature's damage isn't prevented.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P1, "Hunter's Ambush", &[]);
    t.resolve_all();
    t.attack(&[(bears, pl(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn rhonass_monument_cant_target_the_creature_spell() {
    cr!("115.1", "603.3d");
    ruling!(
        "Rhonas's Monument",
        "The triggered ability of Rhonas's Monument can't target the creature spell that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhonas's Monument");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let from = t.asked().len();
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    let candidates: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(candidates.len(), 1);
    assert!(candidates[0].contains(&obj(goblin)));
    assert!(!candidates[0].contains(&obj(bears)));
}

#[test]
fn xenagos_type_change_works_only_on_the_battlefield() {
    cr!("700.5", "611.3a");
    ruling!(
        "Xenagos, God of Revels",
        "The type-changing ability that can make Xenagos not be a creature functions only on the battlefield. It's always a creature card in other zones, regardless of your devotion to red and green. It's always a creature spell while it's on the stack."
    );
    ruling!(
        "Xenagos, God of Revels",
        "Xenagos's abilities function as long as it's on the battlefield, regardless of whether it's a creature."
    );
    let mut t = TestGame::new(2);
    let x = t.hand(P0, "Xenagos, God of Revels");
    t.g.recompute();
    assert!(t.obj_now(x).is(CardType::Creature));
    let gy = t.graveyard(P0, "Xenagos, God of Revels");
    t.g.recompute();
    assert!(t.obj_now(gy).is(CardType::Creature));
    give_mana_for(&mut t, P0, "Xenagos, God of Revels");
    let spell = t.cast(P0, x).go();
    t.g.recompute();
    assert!(t.obj_now(spell).is(CardType::Creature));
    t.resolve_all();
    let x = t.named_on_battlefield("Xenagos, God of Revels")[0];
    assert!(!t.obj_now(x).is(CardType::Creature));
    // Not a creature, its combat trigger still works.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn xenagos_enters_as_a_creature_only_with_enough_devotion_including_its_own_cost() {
    cr!("700.5", "614.12", "603.6a");
    ruling!(
        "Xenagos, God of Revels",
        "When Xenagos enters the battlefield, your devotion to red and green will determine if a creature entered the battlefield or not for abilities that trigger whenever a creature enters the battlefield. The mana symbols in Xenagos's own mana cost are counted when determining this."
    );
    supported("Soul Warden");
    // Kalonian Tusker ({G}{G}), Boggart Brute's {R} and Xenagos's own {R}{G}: 5 is too
    // few; with Ash Zealot ({R}{R}) it's 7.
    for (zealot, life) in [(false, 20), (true, 21)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Soul Warden");
        t.battlefield(P0, "Kalonian Tusker");
        t.battlefield(P0, "Boggart Brute");
        if zealot {
            t.battlefield(P0, "Ash Zealot");
        }
        // Soul Warden: "Whenever another creature enters, you gain 1 life."
        t.enter(P0, "Xenagos, God of Revels");
        t.resolve_all();
        assert_eq!(t.life(P0), life, "zealot: {zealot}");
    }
}

#[test]
fn toxic_iguanar_deathtouch_checked_when_damage_is_dealt() {
    cr!("510.2", "702.2b", "704.3");
    ruling!(
        "Toxic Iguanar",
        "Whether Toxic Iguanar has deathtouch matters only when it deals damage (that is, at the point when its combat damage resolves, or at the point when a spell or ability that causes it to deal damage resolves). For example, if your only green permanent is a creature that receives lethal damage at the same time Toxic Iguanar deals damage to a creature, the deathtouch ability will still apply and the creature Toxic Iguanar dealt damage to will be destroyed."
    );
    supported("Toxic Iguanar");
    let mut t = TestGame::new(2);
    let iguanar = t.battlefield(P0, "Toxic Iguanar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    let ogre = t.battlefield(P1, "Gray Ogre");
    to_blockers(
        &mut t,
        &[(iguanar, pl(P1)), (bears, pl(P1))],
        &[(wall, iguanar), (ogre, bears)],
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(wall));
}

#[test]
fn verdant_succession_search_is_optional() {
    cr!("701.23a", "603.10a");
    ruling!("Verdant Succession", "You can choose not to find the card.");
    supported("Verdant Succession");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verdant Succession");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let copy = t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[]);
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.zone(t.g.current(copy)), Zone::Library(P0));
}

#[test]
fn ivy_seer_can_reveal_zero_cards() {
    cr!("608.2h", "701.20a");
    ruling!("Ivy Seer", "You can reveal zero cards and give +0/+0.");
    supported("Ivy Seer");
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Ivy Seer");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    t.answer_choose(P0, &[]);
    t.activate(P0, seer, 0, &[obj(seer)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(seer), (1, 1));
    assert!(t.obj_now(seer).tapped);
}

#[test]
fn hunt_the_hunter_needs_both_targets() {
    cr!("601.2c", "115.1");
    ruling!(
        "Hunt the Hunter",
        "You must target both a green creature you control and a green creature an opponent controls to cast Hunt the Hunter."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Raging Goblin");
    give_mana_for(&mut t, P0, "Hunt the Hunter");
    let hunt = t.hand(P0, "Hunt the Hunter");
    assert!(t.cast(P0, hunt).target(bears).try_go().is_err());
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(t.cast(P0, hunt).target(bears).target(theirs).try_go().is_ok());
}

#[test]
fn rofelloss_gift_reveals_and_chooses_on_resolution() {
    cr!("608.2", "701.20a");
    ruling!("Rofellos's Gift", "You reveal and you choose cards during the resolution.");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Pacifism");
    t.graveyard(P0, "Pacifism");
    give_mana_for(&mut t, P0, "Rofellos's Gift");
    let gift = t.hand(P0, "Rofellos's Gift");
    t.cast(P0, gift).go();
    // A green card that reaches the hand after the spell was cast can be revealed.
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(a)]);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(a)), Zone::Hand(P0));
    assert_eq!(hand_count(&t, "Pacifism"), 1);
}

fn hand_count(t: &TestGame, name: &str) -> usize {
    t.g.player(P0)
        .hand
        .iter()
        .filter(|c| t.g.obj(**c).chars.name == name)
        .count()
}

#[test]
fn yeva_flash_permission_still_needs_the_mana_cost() {
    cr!("702.8a", "601.2h");
    ruling!(
        "Yeva, Nature's Herald",
        "You still pay the costs for green creature spells you cast."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yeva, Nature's Herald");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    t.lands(P0, "Forest", 2);
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, bears).try_go().is_ok());
    assert_eq!(tapped_lands(&t, P0), 2);
}


