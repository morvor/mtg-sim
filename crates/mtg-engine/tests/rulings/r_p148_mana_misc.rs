//! Rulings batch P148 — other abilities of cards with restricted mana: conditions checked
//! only on activation, values determined on resolution, abilities granted to spells cast
//! from a graveyard (Rivaz of the Claw), mana for costs that contain {X}, delayed triggers
//! that skip mana abilities, and blocking restrictions checked only as blockers are declared.

use crate::r_p076_common::unblockable_after_blocked;
use crate::r_p148_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

/// Wraps P0's agent, recording `probe(game)` whenever a decision other than priority is
/// asked of P0.
struct Spy {
    inner: Box<dyn Agent>,
    probe: Box<dyn Fn(&Game) -> bool + Send>,
    seen: Arc<Mutex<Vec<bool>>>,
}

impl Agent for Spy {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if !matches!(d, Decision::Priority { .. }) {
            self.seen.lock().unwrap().push((self.probe)(g));
        }
        self.inner.decide(g, p, d)
    }
}

fn spy(t: &mut TestGame, probe: impl Fn(&Game) -> bool + Send + 'static) -> Arc<Mutex<Vec<bool>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[0], Box::new(PassiveAgent) as Box<dyn Agent>);
    agents[0] = Box::new(Spy {
        inner,
        probe: Box::new(probe),
        seen: seen.clone(),
    });
    seen
}

// --- Checked on activation / determined on resolution -----------------------------------------

#[test]
fn sliver_hive_checks_for_a_sliver_only_on_activation() {
    cr!("602.5b", "608.2b");
    ruling!(
        "Sliver Hive",
        "Whether you control a Sliver is checked only when you activate the last ability, not as that ability resolves."
    );
    supported("Sliver Hive");
    let mut t = TestGame::new(2);
    let hive = t.battlefield(P0, "Sliver Hive");
    let sliver = t.battlefield(P0, "Muscle Sliver");
    t.lands(P0, "Wastes", 5);
    t.activate(P0, hive, 2, &[]).unwrap();
    destroy(&mut t, sliver);
    t.resolve_all();
    let tokens = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(tokens.len(), 1);
    assert!(t.g.obj(tokens[0]).chars.has_subtype("Sliver"));
    // Without a Sliver it can't be activated.
    t.g.untap(hive);
    for l in t.g.permanents().map(|o| o.id).collect::<Vec<_>>() {
        t.g.untap(l);
    }
    destroy(&mut t, tokens[0]);
    assert!(t.activate(P0, hive, 2, &[]).is_err());
}

#[test]
fn commodore_guff_counts_planeswalkers_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Commodore Guff",
        "The value of X is determined as Commodore Guff's third ability resolves."
    );
    supported("Commodore Guff");
    let mut t = TestGame::new(2);
    let guff = t.battlefield(P0, "Commodore Guff");
    t.activate(P0, guff, 1, &[]).unwrap();
    // A second planeswalker arrives before it resolves.
    t.battlefield(P0, "Jace Beleren");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn lilypad_village_bird_need_not_still_be_there() {
    cr!("602.5b");
    ruling!(
        "Lilypad Village",
        "It doesn’t matter what happens to the Bird, Frog, Otter, or Rat that turn after it enters."
    );
    supported("Lilypad Village");
    let mut t = TestGame::new(2);
    let village = t.battlefield(P0, "Lilypad Village");
    t.lands(P0, "Island", 1);
    assert!(t.activate(P0, village, 2, &[]).is_err());
    t.g.untap(village);
    let bird = crate::r_s02_common::create_token(&mut t, P0, "Bird");
    destroy(&mut t, bird);
    t.activate(P0, village, 2, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn resonating_lute_resolves_after_the_hand_shrinks() {
    cr!("602.5b", "608.2b");
    ruling!(
        "Resonating Lute",
        "Once you've activated Resonating Lute's last ability, it doesn't matter if the number of cards in your hand drops below seven."
    );
    supported("Resonating Lute");
    let mut t = TestGame::new(2);
    let lute = t.battlefield(P0, "Resonating Lute");
    let cards: Vec<ObjectId> = (0..7).map(|_| t.hand(P0, "Island")).collect();
    t.activate(P0, lute, 0, &[]).unwrap();
    t.g.discard(P0, cards[0], None);
    t.g.discard(P0, cards[1], None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 6);
}

#[test]
fn mech_hangar_isnt_crewing() {
    cr!("702.122b", "603.2");
    ruling!(
        "Mech Hangar",
        "Mech Hangar's last ability doesn't count as “crewing” a Vehicle for any ability that would trigger off of a Vehicle becoming crewed."
    );
    supported("Mech Hangar");
    supported("Mobilizer Mech");
    let mut t = TestGame::new(2);
    let hangar = t.battlefield(P0, "Mech Hangar");
    let mech = t.battlefield(P0, "Mobilizer Mech");
    t.lands(P0, "Wastes", 3);
    t.activate(P0, hangar, 2, &[obj(mech)]).unwrap();
    t.resolve_all();
    assert!(t.g.obj(mech).is(mtg_engine::types::CardType::Creature));
    assert_eq!(t.stack_len(), 0);
    // Crewing it does trigger it.
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mobilizer Mech");
    t.battlefield(P0, "Hill Giant");
    activate_containing(&mut t, P0, mech, "Crew").unwrap();
    t.resolve();
    assert!(t.stack_len() >= 1);
}

// --- Blocks checked only as declared ------------------------------------------------------------

#[test]
fn turtle_lair_after_blocks() {
    cr!("509.1h");
    ruling!(
        "Turtle Lair",
        "Once a Ninja or Turtle creature has been blocked, activating Turtle Lair's last ability targeting that creature won't cause it to become unblocked."
    );
    supported("Turtle Lair");
    let mut t = TestGame::new(2);
    let lair = t.battlefield(P0, "Turtle Lair");
    t.lands(P0, "Wastes", 3);
    let ninja = t.battlefield(P0, "Ninja of the Deep Hours");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    unblockable_after_blocked(&mut t, ninja, bears, |t| {
        t.activate(P0, lair, 2, &[obj(ninja)]).unwrap();
    });
}

#[test]
fn jasmine_boreal_checks_abilities_as_blockers_are_declared() {
    cr!("509.1b", "509.1h");
    ruling!(
        "Jasmine Boreal of the Seven",
        "Checking whether a creature has abilities for Jasmine Boreal of the Seven’s last ability happens only once each combat, as blockers are declared."
    );
    supported("Jasmine Boreal of the Seven");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jasmine Boreal of the Seven");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let drake = t.battlefield(P1, "Wind Drake");
    let jump = t.hand(P1, "Jump");
    t.lands(P1, "Island", 1);
    to_combat(&mut t, P0);
    crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    // A creature with abilities can't block it.
    assert!(!legal_blocks(&mut t, P1, &[(drake, bears)]));
    // Blocked by a creature with no abilities, which then gains flying: still blocked.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jasmine Boreal of the Seven");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant2 = t.battlefield(P1, "Hill Giant");
    let jump2 = t.hand(P1, "Jump");
    t.lands(P1, "Island", 1);
    to_combat(&mut t, P0);
    unblockable_after_blocked(&mut t, bears, giant2, |t| {
        t.cast(P1, jump2).target(giant2).go();
    });
    let _ = (giant, jump);
}

#[test]
fn jasmine_boreal_mana_and_spells_given_abilities() {
    cr!("106.6", "113.2");
    ruling!(
        "Jasmine Boreal of the Seven",
        "Some effects give spells a player controls an ability. In that case, mana from Jasmine Boreal of the Seven’s mana ability can’t be spent to cast that spell"
    );
    supported("Silverquill Lecturer");
    for lecturer in [false, true] {
        let mut t = TestGame::new(2);
        let jas = t.battlefield(P0, "Jasmine Boreal of the Seven");
        if lecturer {
            t.battlefield(P0, "Silverquill Lecturer");
        }
        t.activate(P0, jas, 0, &[]).unwrap();
        let bears = t.hand(P0, "Grizzly Bears");
        assert_eq!(t.cast(P0, bears).try_go().is_ok(), !lecturer);
    }
}

// --- Rivaz of the Claw ------------------------------------------------------------------------

/// P0 controls Rivaz and lands for Shivan Dragon ({4}{R}{R}), which is in P0's graveyard.
fn rivaz_setup() -> (TestGame, ObjectId) {
    supported("Rivaz of the Claw");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rivaz of the Claw");
    t.lands(P0, "Mountain", 6);
    let dragon = t.graveyard(P0, "Shivan Dragon");
    (t, dragon)
}

#[test]
fn rivaz_dragon_moves_to_the_stack_as_casting_begins() {
    cr!("601.2a");
    ruling!(
        "Rivaz of the Claw",
        "Once you begin casting a spell from your graveyard, it immediately moves to the stack."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rivaz of the Claw");
    t.lands(P0, "Mountain", 7);
    let verix = t.graveyard(P0, "Verix Bladewing");
    let seen = spy(&mut t, |g| {
        g.player(P0).graveyard.is_empty() && g.stack.len() == 1
    });
    t.cast(P0, verix).kicked(true).go();
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty(), "no choice asked while casting");
    assert!(seen.iter().all(|x| *x), "{seen:?}");
}

#[test]
fn rivaz_dragon_cast_from_the_graveyard_is_exiled_when_it_dies() {
    cr!("113.6", "603.6c");
    ruling!(
        "Rivaz of the Claw",
        "The last ability triggers when you cast a Dragon creature spell from your graveyard for any reason. It triggers even if you didn’t use mana generated by Rivaz’s activated ability to pay its costs."
    );
    ruling!(
        "Rivaz of the Claw",
        "The ability that the spell gains as Rivaz’s last ability resolves continues to apply to the permanent that spell becomes after the spell resolves, even if it stops being a Dragon or stops being a creature."
    );
    supported("Unnatural Selection");
    let (mut t, dragon) = rivaz_setup();
    t.cast(P0, dragon).go();
    t.resolve_all();
    let d = t.named_on_battlefield("Shivan Dragon")[0];
    // It stops being a Dragon.
    let sel = t.battlefield(P0, "Unnatural Selection");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, sel, 0, &[obj(d)]).unwrap();
    t.resolve_all();
    assert!(!t.g.obj(d).chars.has_subtype("Dragon"));
    destroy(&mut t, d);
    t.resolve_all();
    assert!(t.in_exile("Shivan Dragon"));
}

#[test]
fn rivaz_countered_dragon_isnt_exiled() {
    cr!("113.6", "701.6a");
    ruling!(
        "Rivaz of the Claw",
        "The ability that the spell gains only applies to it once it’s on the battlefield. If the spell is countered before it resolves, that ability will not exile it."
    );
    let (mut t, dragon) = rivaz_setup();
    let spell = t.g.current(dragon);
    t.cast(P0, spell).go();
    t.resolve(); // Rivaz's trigger.
    let spell = *t.g.stack.last().unwrap();
    let cancel = t.hand(P1, "Cancel");
    t.lands(P1, "Island", 3);
    t.cast(P1, cancel).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shivan Dragon"));
}

#[test]
fn rivaz_doesnt_change_when_the_dragon_may_be_cast() {
    cr!("307.1", "601.3");
    ruling!(
        "Rivaz of the Claw",
        "The casting permission granted by Rivaz’s third ability doesn’t change when you may cast the spell from your graveyard."
    );
    let (mut t, dragon) = rivaz_setup();
    assert!(!legal_cast_methods(&mut t, P0, dragon).is_empty());
    // On P0's turn with a spell on the stack: no.
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    assert!(legal_cast_methods(&mut t, P0, dragon).is_empty());
    t.resolve_all();
    // During P1's turn: no.
    t.set_step(P1, Step::PrecombatMain);
    assert!(legal_cast_methods(&mut t, P0, dragon).is_empty());
}

// --- Dalakos ---------------------------------------------------------------------------------

#[test]
fn dalakos_creature_equipment_isnt_equipped() {
    cr!("301.5", "702.151b");
    ruling!(
        "Dalakos, Crafter of Wonders",
        "An equipped creature is one with one or more Equipment attached. A creature that’s also an Equipment isn’t an equipped creature."
    );
    supported("Lizard Blades");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dalakos, Crafter of Wonders");
    let blades = t.battlefield(P0, "Lizard Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(!t.g.obj(blades).chars.has_keyword(KeywordKind::Flying));
    assert!(!t.g.obj(bears).chars.has_keyword(KeywordKind::Flying));
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, blades, "Reconfigure").unwrap();
    t.resolve_all();
    t.g.recompute();
    let b = t.g.obj(bears);
    assert!(b.chars.has_keyword(KeywordKind::Flying) && b.chars.has_keyword(KeywordKind::Haste));
}

// --- Pit Automaton ---------------------------------------------------------------------------

#[test]
fn pit_automaton_skips_exhaust_mana_abilities() {
    cr!("603.7c", "702.177a");
    ruling!(
        "Pit Automaton",
        "the delayed triggered ability doesn’t trigger if you activate an exhaust ability that is also a mana ability."
    );
    supported("Pit Automaton");
    supported("Loot, the Pathfinder");
    let mut t = TestGame::new(2);
    let pit = t.battlefield(P0, "Pit Automaton");
    let loot = t.battlefield(P0, "Loot, the Pathfinder");
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.activate(P0, pit, 1, &[]).unwrap();
    t.resolve_all();
    // The exhaust mana ability: no copy.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    activate_containing(&mut t, P0, loot, "Add three").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool_total(&t, P0), 3);
    // The next (non-mana) exhaust ability is copied.
    t.g.untap(loot);
    activate_containing(&mut t, P0, loot, "Draw three").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 6);
}

// --- Mana for costs that contain {X} -------------------------------------------------------------

/// P0's pool holds four colorless mana that may be spent only on costs that contain {X},
/// from `source` (Rosheen Meanderer, or two basic lands with Nexos's ability).
fn x_mana(source: &str) -> TestGame {
    supported(source);
    let mut t = TestGame::new(2);
    if source == "Nexos" {
        t.battlefield(P0, "Nexos");
        for p in t.lands(P0, "Plains", 2) {
            t.activate(P0, p, 1, &[]).unwrap();
        }
    } else {
        let r = t.battlefield(P0, source);
        t.activate(P0, r, 0, &[]).unwrap();
    }
    assert_eq!(pool_total(&t, P0), 4);
    assert!(t
        .g
        .player(P0)
        .mana_pool
        .mana
        .iter()
        .all(|m| m.restriction.is_some()));
    t
}

/// The mana pays a part of a cost that isn't X (Consume Spirit's {1}, with X = 0 and "spend
/// only black mana on X"), and the rest of it pays a different cost (Hangarback Walker with
/// X = 1); it can't pay a cost without {X} (Bonesplitter).
fn x_mana_spends(source: &str) {
    let mut t = x_mana(source);
    let splitter = t.hand(P0, "Bonesplitter");
    assert!(t.cast(P0, splitter).try_go().is_err(), "{source}");
    t.lands(P0, "Swamp", 1);
    let spirit = t.hand(P0, "Consume Spirit");
    t.cast(P0, spirit).x(0).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 3, "{source}");
    let walker = t.hand(P0, "Hangarback Walker");
    t.cast(P0, walker).x(1).go();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 1, "{source}");
    assert_eq!(t.counters(walker, counters::PLUS1), 1);
}

#[test]
fn rosheen_meanderer_x_costs() {
    cr!("106.6", "107.3", "601.2f");
    ruling!(
        "Rosheen Meanderer",
        "You can spend mana generated by Rosheen on a cost that includes {X} even if you’ve chosen an X of 0, or if the card specifies that you can spend only colored mana on X."
    );
    ruling!(
        "Rosheen Meanderer",
        "You can spend mana generated by Rosheen on any part of a cost that contains {X}. You’re not limited to spending it only on the {X} part."
    );
    ruling!(
        "Rosheen Meanderer",
        "You don’t have to spend all four mana on the same cost."
    );
    x_mana_spends("Rosheen Meanderer");
}

#[test]
fn nexos_x_costs() {
    cr!("106.6", "107.3", "601.2f");
    ruling!(
        "Nexos",
        "You can spend mana generated by the granted ability on a cost that includes {X} even if you've chosen an X of 0"
    );
    ruling!(
        "Nexos",
        "You can spend mana generated by the granted ability on any part of a cost that contains {X}. You're not limited to spending it only on the {X} part."
    );
    ruling!(
        "Nexos",
        "You don't have to spend all of the mana on the same cost."
    );
    x_mana_spends("Nexos");
}

#[test]
fn cost_that_contains_x_includes_activation_and_additional_costs() {
    cr!("106.6", "107.3", "601.2f", "602.2b");
    ruling!(
        "Nexos",
        "A \"cost that contains {X}\" may be a spell's total cost, an activated ability's cost"
    );
    ruling!(
        "Rosheen Meanderer",
        "A “cost that contains {X}” may be a spell’s total cost, an activated ability’s cost"
    );
    for source in ["Nexos", "Rosheen Meanderer"] {
        // An activated ability's cost: Mirror Entity's {X}.
        let mut t = x_mana(source);
        let entity = t.battlefield(P0, "Mirror Entity");
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        t.activate(P0, entity, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.pt(entity), (2, 2), "{source}");
        // A spell's total cost including its kicker {X} (Emblazoned Golem, {2} kicker {X},
        // X paid with colored mana): the {2} too.
        let mut t = x_mana(source);
        t.lands(P0, "Mountain", 1);
        let golem = t.hand(P0, "Emblazoned Golem");
        t.cast(P0, golem).kicked(true).x(1).go();
        t.resolve_all();
        assert_eq!(t.counters(golem, counters::PLUS1), 1, "{source}");
        assert_eq!(pool_total(&t, P0), 2, "{source}");
    }
}

// --- Altar of the Lost ---------------------------------------------------------------------------

#[test]
fn altar_of_the_lost_flashback_spells_cast_from_a_graveyard() {
    cr!("106.6", "702.34a");
    ruling!(
        "Altar of the Lost",
        "You can spend mana produced by Altar of the Lost to cast any spell with flashback that you cast from a graveyard. You don’t have to be using flashback to cast that spell"
    );
    ruling!(
        "Altar of the Lost",
        "Altar of the Lost doesn’t allow you to cast spells from any other player’s graveyard"
    );
    supported("Altar of the Lost");
    supported("Kess, Dissident Mage");
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Altar of the Lost");
    t.battlefield(P0, "Kess, Dissident Mage");
    let think = t.graveyard(P0, "Think Twice");
    let theirs = t.graveyard(P1, "Think Twice");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, altar, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 2);
    assert!(legal_cast_methods(&mut t, P0, theirs).is_empty());
    // Kess's permission ({1}{U}), not flashback ({2}{U}).
    let methods = legal_cast_methods(&mut t, P0, think);
    let m = methods
        .into_iter()
        .find(|m| *m != CastMethod::Keyword(KeywordKind::Flashback))
        .expect("Kess's permission");
    t.cast(P0, think).method(m).go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.zone(think), Zone::Exile);
}

#[test]
fn crucible_mana_cant_turn_a_face_down_dragon_face_up() {
    cr!("106.6", "116.2b", "702.37e");
    ruling!(
        "Crucible of the Spirit Dragon",
        "Notably, turning a face-down creature face up isn't an activated ability."
    );
    supported("Imperial Hellkite");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let kite = t.hand(P0, "Imperial Hellkite");
    t.cast(P0, kite)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let crucible = t.battlefield(P0, "Crucible of the Spirit Dragon");
    t.g.add_counters(Entity::Object(crucible), "storage", 8, None);
    let kite = t.g.current(kite);
    assert!(t.g.obj(kite).face_down);
    t.answer(P0, DecisionKind::X, Answer::Number(8));
    for _ in 0..8 {
        t.answer(P0, DecisionKind::Option, Answer::Index(3));
    }
    t.activate(P0, crucible, 2, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 8);
    let up =
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::TurnFaceUp {
            obj: kite,
        });
    t.g.turn.priority = Some(P0);
    assert!(t.g.perform_action(P0, up).is_err());
    assert!(t.g.obj(kite).face_down);
    assert_eq!(pool_total(&t, P0), 8);
    // Unrestricted mana can.
    empty_pool(&mut t, P0);
    mana(&mut t, P0, mtg_engine::mana::ManaType::R, 8);
    let up =
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::TurnFaceUp {
            obj: kite,
        });
    t.g.perform_action(P0, up).unwrap();
    assert!(!t.g.obj(t.g.current(kite)).face_down);
}
