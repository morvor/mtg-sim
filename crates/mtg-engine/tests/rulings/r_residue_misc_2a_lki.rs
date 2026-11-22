//! Item `rulings-residue-misc-2`, pass A — last known information and objects that leave
//! or change while an ability waits on the stack (CR 608.2h, 113.7a, 603.10, 611.2b):
//! Grenzo, Pyrogoyf, Rip, Squall, Old Man of the Sea, Opportunistic Dragon, Éowyn,
//! Tawnos's Coffin and Wonderscape Sage.

use crate::r_residue_misc_2a_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

// --- Grenzo, Dungeon Warden ----------------------------------------------------------------

/// P0's Grenzo (a 2/2, with `counters` +1/+1 counters) and `bottom` as the bottom card of
/// P0's library; Grenzo's ability is activated and on the stack.
fn grenzo_with(bottom: &str, counters: u32) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Grenzo, Dungeon Warden");
    plus1(&mut t, g, counters);
    let card = t.library_top(P0, bottom);
    to_bottom(&mut t, P0, card);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, g, 0, &[]).expect("Grenzo's ability");
    (t, g, card)
}

#[test]
fn grenzo_compares_its_power_as_the_ability_resolves_or_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Grenzo, Dungeon Warden",
        "Compare Grenzo's power when the ability resolves with the power of the creature card in your graveyard to determine if you put it onto the battlefield. If Grenzo isn't on the battlefield at this time, use its last known power from when it was on the battlefield."
    );
    supported("Grenzo, Dungeon Warden");
    // Baseline: a 2/2 Grenzo doesn't put a Hill Giant (3 power) onto the battlefield.
    let (mut t, _, _) = grenzo_with("Hill Giant", 0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // Made a 3/3 after activating, as the ability resolves: it does.
    let (mut t, g, _) = grenzo_with("Hill Giant", 0);
    plus1(&mut t, g, 1);
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Hill Giant").len(),
        1,
        "{}",
        t.dump_log()
    );
    // A 3/3 Grenzo destroyed in response: its last known power (3) is used.
    let (mut t, g, _) = grenzo_with("Hill Giant", 1);
    destroy(&mut t, g);
    assert!(t.in_graveyard(P0, "Grenzo, Dungeon Warden"));
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Hill Giant").len(),
        1,
        "{}",
        t.dump_log()
    );
}

// --- Pyrogoyf -------------------------------------------------------------------------------

#[test]
fn pyrogoyf_uses_the_last_known_power_of_the_creature_that_left() {
    cr!("608.2h", "113.7a", "604.3");
    ruling!(
        "Pyrogoyf",
        "If Pyrogoyf or the relevant Lhurgoyf creature leaves the battlefield while Pyrogoyf's last ability is on the stack, use its power as it last existed on the battlefield to determine how much damage is dealt."
    );
    supported("Pyrogoyf");
    supported("Tarmogoyf");
    // Pyrogoyf itself: only a land card in graveyards, so it's a 1/2; once it dies, a
    // creature card is there too (2 card types), but it deals 1 damage.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Forest");
    t.answer_targets(P0, &[pl(P1)]);
    let goyf = t.enter(P0, "Pyrogoyf");
    t.settle();
    assert_eq!(t.pt(goyf), (1, 2));
    destroy(&mut t, goyf);
    assert!(t.in_graveyard(P0, "Pyrogoyf"));
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
    // Another Lhurgoyf (Tarmogoyf) entering, then dying in response: its last known power.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Forest");
    t.battlefield(P0, "Pyrogoyf");
    t.answer_targets(P0, &[pl(P1)]);
    let tarmo = t.enter(P0, "Tarmogoyf");
    t.settle();
    assert_eq!(t.pt(tarmo).0, 1);
    destroy(&mut t, tarmo);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
}

// --- Rip, Spawn Hunter -----------------------------------------------------------------------

#[test]
fn rip_that_left_tapped_uses_its_last_known_power() {
    cr!("608.2h", "603.4", "113.7a");
    ruling!(
        "Rip, Spawn Hunter",
        "If Rip leaves the battlefield before its ability resolves—and it was still tapped as it left—use its power as it last existed on the battlefield to determine the value of X."
    );
    supported("Rip, Spawn Hunter");
    let mut t = TestGame::new(2);
    let rip = t.battlefield(P0, "Rip, Spawn Hunter");
    t.g.tap(rip);
    let cards = stack_library(
        &mut t,
        P0,
        &[
            "Llanowar Elves",
            "Grizzly Bears",
            "Hill Giant",
            "Air Elemental",
            "Craw Wurm",
        ],
    );
    t.set_step(P0, Step::EndOfCombat);
    t.advance_to(P0, Step::PostcombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1, "Rip's ability triggers");
    destroy(&mut t, rip);
    assert!(t.in_graveyard(P0, "Rip, Spawn Hunter"));
    // Rip was a 4/4: the top four cards are revealed.
    let four: Vec<Entity> = cards[..4].iter().map(|c| obj(*c)).collect();
    t.answer_choose(P0, &four);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4, "{}", t.dump_log());
    assert!(!t.in_hand(P0, "Craw Wurm"));
}

// --- Squall, Gunblade Duelist ----------------------------------------------------------------

#[test]
fn squall_that_left_deals_damage_equal_to_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Squall, Gunblade Duelist",
        "If Squall isn't on the battlefield as its last ability resolves, use its power as it last existed on the battlefield to determine how much damage the ability deals."
    );
    supported("Squall, Gunblade Duelist");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let squall = t.enter(P0, "Squall, Gunblade Duelist");
    t.settle();
    plus1(&mut t, squall, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert_eq!(t.stack_len(), 1, "Squall's ability triggers");
    destroy(&mut t, squall);
    assert!(t.in_graveyard(P0, "Squall, Gunblade Duelist"));
    t.resolve_all();
    // Squall was a 4/3 (a +1/+1 counter): 4 damage.
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
}

// --- Old Man of the Sea ------------------------------------------------------------------------

/// P0's Old Man of the Sea (2/3) activates its ability targeting P1's Grizzly Bears (2/2).
fn old_man() -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let man = t.battlefield(P0, "Old Man of the Sea");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, man, 0, &[obj(bears)])
        .expect("Old Man of the Sea's ability");
    (t, man, bears)
}

#[test]
fn old_man_of_the_sea_gains_nothing_if_untapped_or_outmatched_before_resolution() {
    cr!("611.2b", "613.1b");
    ruling!(
        "Old Man of the Sea",
        "If Old Man of the Sea stops being tapped before its ability resolves — even if it becomes tapped again right away — you won’t gain control of the targeted creature at all. The same is true if the targeted creature’s power becomes greater than Old Man of the Sea’s power before the ability resolves."
    );
    supported("Old Man of the Sea");
    // Baseline: control is gained.
    let (mut t, _, bears) = old_man();
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0);
    // Untapped and tapped again in response: no control at all.
    let (mut t, man, bears) = old_man();
    t.g.untap(man);
    t.g.tap(man);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1, "{}", t.dump_log());
    // The Bears made a 3/3 in response: no control, and none later when it shrinks.
    let (mut t, _, bears) = old_man();
    plus1(&mut t, bears, 1);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1, "{}", t.dump_log());
}

// --- Opportunistic Dragon ----------------------------------------------------------------------

/// P1's Pitiless Plunderer (a Human) and P0's Opportunistic Dragon entering, its ability
/// targeting the Plunderer on the stack.
fn dragon() -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let plunderer = t.battlefield(P1, "Pitiless Plunderer");
    t.answer_targets(P0, &[obj(plunderer)]);
    let d = t.enter(P0, "Opportunistic Dragon");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (t, d, plunderer)
}

#[test]
fn opportunistic_dragon_that_left_before_resolution_takes_nothing() {
    cr!("611.2b", "610.3");
    ruling!(
        "Opportunistic Dragon",
        "If Opportunistic Dragon leaves the battlefield before its triggered ability resolves, your opponent keeps control of their Human or artifact, it doesn't lose any abilities, and it can still attack or block as normal. You don't gain control of it at all."
    );
    supported("Opportunistic Dragon");
    supported("Pitiless Plunderer");
    let (mut t, d, plunderer) = dragon();
    let abilities = n_abilities(&mut t, plunderer);
    destroy(&mut t, d);
    t.resolve_all();
    assert_eq!(controller(&t, plunderer), P1);
    assert_eq!(n_abilities(&mut t, plunderer), abilities);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, plunderer));
}

#[test]
fn opportunistic_dragon_s_stolen_permanent_stays_suppressed_under_another_controller() {
    cr!("613.1b", "613.1f", "611.2b");
    ruling!(
        "Opportunistic Dragon",
        "If another player gains control of the permanent Opportunistic Dragon has kidnapped, that permanent doesn't regain its abilities and still can't attack or block."
    );
    let (mut t, _, plunderer) = dragon();
    t.resolve_all();
    assert_eq!(controller(&t, plunderer), P0);
    assert_eq!(n_abilities(&mut t, plunderer), 0);
    // P1 gains control of it back by a later effect.
    give_control(&mut t, plunderer, P1);
    assert_eq!(controller(&t, plunderer), P1);
    assert_eq!(n_abilities(&mut t, plunderer), 0);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, plunderer));
    let attacker = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(attacker, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(plunderer, attacker)]));
    assert!(legal_blocks(&mut t, P1, &[]));
}

#[test]
fn opportunistic_dragon_dying_doesnt_trigger_the_stolen_permanent() {
    cr!("603.10a", "613.1f");
    ruling!(
        "Opportunistic Dragon",
        "If the stolen permanent normally has any abilities that would trigger when Opportunistic Dragon dies, those abilities won't trigger."
    );
    let (mut t, d, plunderer) = dragon();
    t.resolve_all();
    assert_eq!(controller(&t, plunderer), P0);
    destroy(&mut t, d);
    t.resolve_all();
    // The Plunderer goes back to P1 with its ability, but nothing triggered for the Dragon.
    assert_eq!(controller(&t, plunderer), P1);
    assert!(n_abilities(&mut t, plunderer) > 0);
    assert!(tokens(&t, P0).is_empty() && tokens(&t, P1).is_empty());
    // Baseline: the Plunderer, unstolen, does trigger on another creature of P1's dying.
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens(&t, P1).len(), 1);
}

// --- Éowyn, Fearless Knight --------------------------------------------------------------------

#[test]
fn eowyn_uses_the_exiled_creature_s_colors_as_it_last_existed() {
    cr!("608.2h", "702.16a");
    ruling!(
        "Éowyn, Fearless Knight",
        "Éowyn, Fearless Knight cares about the creature's colors just before it left the battlefield. These could be different from the creature card's colors in exile (for example, if it had a copy effect applied to it on the battlefield.)"
    );
    supported("Éowyn, Fearless Knight");
    let mut t = TestGame::new(2);
    let shivan = t.battlefield(P1, "Shivan Dragon");
    let clone = clone_of(&mut t, P1, shivan);
    t.answer_targets(P0, &[obj(clone)]);
    let eowyn = t.enter(P0, "Éowyn, Fearless Knight");
    t.resolve_all();
    assert!(t.in_exile("Clone"));
    // Protection from red (the copy's color), not from blue (Clone's own).
    let o = t.obj_now(eowyn);
    let protection: Vec<_> = o
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            mtg_engine::ability::AbilityKind::Keyword(k)
                if k.kind == mtg_engine::keywords::KeywordKind::Protection =>
            {
                k.filter.as_ref().map(|f| format!("{f:?}"))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        protection,
        vec![format!(
            "{:?}",
            mtg_engine::ability::Filter::Or(vec![mtg_engine::ability::Filter::Color(Color::Red)])
        )]
    );
    // A red spell can't target it; a blue one can.
    let from = t.asked().len();
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    let _ = t.cast(P1, bolt).target(pl(P0)).try_go();
    let ray = t.hand(P1, "Unsummon");
    t.lands(P1, "Island", 1);
    let _ = t.cast(P1, ray).target(eowyn).try_go();
    let cands = crate::r_s02_common::target_candidates(&t, P1, from);
    assert_eq!(cands.len(), 2);
    assert!(!cands[0].contains(&obj(eowyn)));
    assert!(cands[1].contains(&obj(eowyn)));
}

// --- Tawnos's Coffin -----------------------------------------------------------------------------

#[test]
fn tawnos_s_coffin_returns_a_card_that_was_a_creature_only_on_the_battlefield() {
    cr!("400.7", "610.3c");
    ruling!(
        "Tawnos's Coffin",
        "The effect doesn’t care what types the card has after it is exiled, only that it have been a creature while on the battlefield."
    );
    supported("Tawnos's Coffin");
    supported("Mutavault");
    let mut t = TestGame::new(2);
    let coffin = t.battlefield(P0, "Tawnos's Coffin");
    let vault = t.battlefield(P1, "Mutavault");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, vault, 1, &[]).expect("Mutavault animates");
    t.resolve_all();
    assert!(t.obj_now(vault).is(CardType::Creature));
    t.lands(P0, "Wastes", 3);
    t.activate(P0, coffin, 0, &[obj(vault)])
        .expect("Tawnos's Coffin's ability");
    t.resolve_all();
    assert!(t.in_exile("Mutavault"));
    // A land card in exile; the Coffin leaving returns it.
    destroy(&mut t, coffin);
    t.resolve_all();
    let back = t.named_on_battlefield("Mutavault");
    assert_eq!(back.len(), 1, "{}", t.dump_log());
    assert!(t.obj_now(back[0]).tapped);
    assert_eq!(t.zone(back[0]), Zone::Battlefield);
}

// --- Wonderscape Sage ----------------------------------------------------------------------------

/// P0's Wonderscape Sage activates returning `land` (P0's) to hand; returns how many cards
/// P0 discarded.
fn sage_returning(t: &mut TestGame, land: ObjectId) -> usize {
    let sage = t.battlefield(P0, "Wonderscape Sage");
    t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(land)]);
    t.activate(P0, sage, 0, &[])
        .expect("Wonderscape Sage's ability");
    let gy = t.graveyard_size(P0);
    t.resolve_all();
    t.graveyard_size(P0) - gy
}

#[test]
fn wonderscape_sage_uses_the_returned_land_s_last_known_subtypes() {
    cr!("608.2h", "205.3i", "707.2");
    ruling!(
        "Wonderscape Sage",
        "Use the subtypes of the returned land as it last existed on the battlefield to determine whether or not you must discard a card. For example, if you return a Copy Land that's a copy of Urza's Cave (a land with the types Urza's and Cave) to your hand to pay the cost of Wonderscape Sage's last ability, you won't discard a card."
    );
    supported("Wonderscape Sage");
    supported("Vesuva");
    // Baseline: a basic Forest (a basic land type only): discard.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    assert_eq!(sage_returning(&mut t, forest), 1);
    // Vesuva copying Urza's Cave: in hand it has no subtypes, but no discard.
    let mut t = TestGame::new(2);
    let cave = t.battlefield(P1, "Urza's Cave");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(cave)]);
    let vesuva = t.enter(P0, "Vesuva");
    t.settle();
    let vesuva = t.g.current(vesuva);
    assert!(t.obj(vesuva).chars.subtypes.iter().any(|s| s == "Cave"));
    assert_eq!(sage_returning(&mut t, vesuva), 0, "{}", t.dump_log());
    assert!(t.in_hand(P0, "Vesuva"));
}

#[test]
fn wonderscape_sage_cares_about_a_nonbasic_land_type_not_a_nonbasic_land() {
    cr!("205.3i", "608.2h");
    ruling!(
        "Wonderscape Sage",
        "Wonderscape Sage's last ability doesn't care if the returned land was nonbasic. It only cares if the land had a nonbasic land type. For example, if you control Omo, Queen of Vesuva and return a Forest with an everything counter on it to your hand to pay the cost of Wonderscape Sage's last ability, you won't discard a card."
    );
    supported("Omo, Queen of Vesuva");
    // A nonbasic land with no land type (Mutavault): discard.
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Mutavault");
    assert_eq!(sage_returning(&mut t, vault), 1);
    // A basic Forest with an everything counter under Omo: every land type, no discard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omo, Queen of Vesuva");
    let forest = t.battlefield(P0, "Forest");
    t.g.add_counters(obj(forest), "everything", 1, None);
    t.g.recompute();
    assert_eq!(sage_returning(&mut t, forest), 0, "{}", t.dump_log());
}
