//! Rulings batch P102 — odd cards: Panglacial Wurm (cast while searching), Aven
//! Mindcensor, Gruul Charm, Oubliette's phasing and "for as long as" (CR 702.26d),
//! Meandering Towershell, Werebear, Phantasmal Dreadmaw, Platinum Angel and Hazoret the
//! Fervent (also in Two-Headed Giant).

use crate::r_p102_common::*;
use crate::r_s01_common::{attack_with, stack_library, with_subtype};
use crate::r_s02_common::destroy;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Panglacial Wurm ---------------------------------------------------------------

/// P0 casts Worldly Tutor ("Search your library for a creature card, reveal it, then
/// shuffle and put that card on top.") with `lands` Forests, answering yes to casting
/// each Wurm.
fn tutor_with_wurms(t: &mut TestGame, lands: usize, wurms: usize) -> Vec<ObjectId> {
    supported("Panglacial Wurm");
    let ws: Vec<ObjectId> = (0..wurms)
        .map(|_| t.library_top(P0, "Panglacial Wurm"))
        .collect();
    t.lands(P0, "Forest", lands);
    let wt = t.hand(P0, "Worldly Tutor");
    t.cast(P0, wt).go();
    for _ in 0..wurms {
        t.answer_yes(P0, true);
    }
    ws
}

#[test]
fn panglacial_wurm_cast_while_searching_isnt_found() {
    cr!("701.23a", "601.2");
    ruling!(
        "Panglacial Wurm",
        "A Panglacial Wurm that you cast while searching your library can't be found by the search effect."
    );
    ruling!(
        "Panglacial Wurm",
        "If you want to cast Panglacial Wurm while searching your library, you must do so before you find any cards with the search effect."
    );
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let ws = tutor_with_wurms(&mut t, 8, 1);
    let from = t.asked().len();
    choose(&mut t, P0, &[bears]);
    t.resolve();
    let asked = t.asked();
    let yes = asked[from..]
        .iter()
        .position(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .unwrap();
    let pick = asked[from..]
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .unwrap();
    assert!(yes < pick, "asked to cast it before finding a card");
    let offered = &entity_choices(&t, P0, from)[0];
    assert!(!offered.contains(&o(ws[0])));
    assert!(!offered.iter().any(|e| e
        .object()
        .is_some_and(|id| t.obj(id).chars.name == "Panglacial Wurm")));
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.player(P0).library[t.library_size(P0) - 1], bears);
}

#[test]
fn panglacial_wurm_pays_its_costs_and_several_can_be_cast() {
    cr!("601.2", "601.2g", "601.2h");
    ruling!(
        "Panglacial Wurm",
        "Casting Panglacial Wurm while searching your library follows all the normal rules for casting a creature spell"
    );
    ruling!(
        "Panglacial Wurm",
        "If you have enough mana, you can cast multiple Panglacial Wurms during a single search effect."
    );
    // Seven Forests left after the Tutor: one Wurm.
    let mut t = TestGame::new(2);
    tutor_with_wurms(&mut t, 8, 1);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 8);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Panglacial Wurm").len(), 1);
    // Six left: not enough; the Wurm stays in the library.
    let mut t = TestGame::new(2);
    let ws = tutor_with_wurms(&mut t, 7, 1);
    t.resolve_all();
    assert!(t.named_on_battlefield("Panglacial Wurm").is_empty());
    assert_eq!(t.zone(ws[0]), Zone::Library(P0));
    // Fifteen: two Wurms.
    let mut t = TestGame::new(2);
    tutor_with_wurms(&mut t, 15, 2);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Panglacial Wurm").len(), 2);
}

// --- Aven Mindcensor ---------------------------------------------------------------

/// P1's Aven Mindcensor; P0's library from the top: three fillers... `top_first`.
fn mindcensor(t: &mut TestGame, top_first: &[&str]) -> Vec<ObjectId> {
    supported("Aven Mindcensor");
    t.battlefield(P1, "Aven Mindcensor");
    stack_library(t, P0, top_first)
}

#[test]
fn aven_mindcensor_limits_the_search_to_the_top_four_then_shuffles() {
    cr!("701.23a", "701.24a", "614.1a");
    ruling!(
        "Aven Mindcensor",
        "Each search works exactly the same as normal except that the opponent doing the searching sees only the top four cards of that library."
    );
    ruling!(
        "Aven Mindcensor",
        "After the search is complete, the entire library is shuffled."
    );
    let mut t = TestGame::new(2);
    let lib = mindcensor(
        &mut t,
        &["Shock", "Forest", "Island", "Swamp", "Lightning Bolt"],
    );
    lots(&mut t, P0);
    let dt = t.hand(P0, "Demonic Tutor");
    t.cast(P0, dt).go();
    let from = t.asked().len();
    choose(&mut t, P0, &[lib[4]]);
    t.resolve();
    let offered = &entity_choices(&t, P0, from)[0];
    assert!(offered.contains(&o(lib[0])));
    assert!(!offered.contains(&o(lib[4])), "not in the top four");
    assert!(!t.in_hand(P0, "Lightning Bolt"));
    assert!(t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Shuffled { player } if *player == P0)));
}

#[test]
fn aven_mindcensor_search_still_counts_as_searching() {
    cr!("701.23a");
    ruling!(
        "Aven Mindcensor",
        "For effects that check whether a player searched a library or may search a library, searching the top four cards of that library counts as searching that library."
    );
    // P1's Ob Nixilis, Unshackled: "Whenever an opponent searches their library, that
    // player sacrifices a creature and loses 10 life."
    let mut t = TestGame::new(2);
    mindcensor(&mut t, &["Shock"]);
    t.battlefield(P1, "Ob Nixilis, Unshackled");
    lots(&mut t, P0);
    let dt = t.hand(P0, "Demonic Tutor");
    t.cast(P0, dt).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 10);
}

#[test]
fn aven_mindcensor_top_four_stay_the_same_during_one_search() {
    cr!("701.23a");
    ruling!(
        "Aven Mindcensor",
        "If an effect uses the word \"search\" once but allows an opponent to search for multiple cards, the top four cards remain the same throughout the entire search."
    );
    // Cultivate ("up to two basic land cards"): one Forest in the top four, another fifth.
    let mut t = TestGame::new(2);
    let lib = mindcensor(&mut t, &["Forest", "Shock", "Shock", "Shock", "Forest"]);
    lots(&mut t, P0);
    let c = t.hand(P0, "Cultivate");
    t.cast(P0, c).go();
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[o(lib[0]), o(lib[4])]);
    t.answer_choose_moved(P0, &[o(lib[0])]);
    t.resolve_all();
    for offered in entity_choices(&t, P0, from) {
        assert!(!offered.contains(&o(lib[4])), "{offered:?}");
    }
    assert_eq!(t.zone(lib[4]), Zone::Library(P0));
}

// --- Gruul Charm --------------------------------------------------------------------

fn gruul_charm(t: &mut TestGame, mode: usize) {
    supported("Gruul Charm");
    lots(t, P0);
    let c = t.hand(P0, "Gruul Charm");
    t.cast(P0, c).modes(&[mode]).go();
    t.resolve_all();
}

#[test]
fn gruul_charm_cant_block_covers_later_and_changed_creatures() {
    cr!("611.3a", "509.1b");
    ruling!(
        "Gruul Charm",
        "If you choose the first mode, no creature without flying will be able to block that turn, including creatures that lose flying and creatures without flying that enter after Gruul Charm resolves."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let atk = t.battlefield(P0, "Hill Giant");
    gruul_charm(&mut t, 0);
    let later = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let frog = t.hand(P0, "Turn to Frog");
    t.cast(P0, frog).target(o(angel)).go();
    t.resolve_all();
    attack_with(&mut t, &[(atk, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(later, atk)]));
    assert!(!legal_blocks(&mut t, P1, &[(angel, atk)]));
}

#[test]
fn gruul_charm_regains_your_permanents_and_tokens_you_own() {
    cr!("111.2", "613.2", "613.7");
    ruling!(
        "Gruul Charm",
        "If you choose the second mode, Gruul Charm's effect will override any other control-changing effects on those permanents."
    );
    ruling!(
        "Gruul Charm",
        "A token is owned by the player under whose control it entered."
    );
    // P1's Control Magic took P0's Bears; P1's Act of Treason took P0's Soldier token; a
    // Spirit token P0's effect created under P1's control is P1's.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P1, "Control Magic", bears);
    assert_eq!(t.obj_now(bears).controller, P1);
    let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    crate::r_p015_common::gain_control(&mut t, P1, soldier);
    let spirit = crate::r_s02_common::create_token(&mut t, P1, "Spirit");
    gruul_charm(&mut t, 1);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(soldier).controller, P0);
    assert_eq!(t.obj_now(spirit).controller, P1);
    assert_eq!(t.obj_now(spirit).owner, P1);
}

// --- Oubliette ----------------------------------------------------------------------

#[test]
fn phasing_out_ends_for_as_long_as_effects() {
    cr!("702.26d", "611.2b");
    ruling!(
        "Oubliette",
        "Any continuous effects with a “for as long as” duration such as that of Awakener Druid ignore phased-out objects."
    );
    supported("Oubliette");
    // P1's Awakener Druid made P1's Forest a 4/5 creature "for as long as this creature
    // remains on the battlefield". Oubliette phases the Druid out: the effect ends, and
    // stays ended when the Druid phases back in.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P1, &[o(forest)]);
    let druid = t.enter(P1, "Awakener Druid");
    t.resolve_all();
    assert!(t.obj_now(forest).is(CardType::Creature));
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[o(druid)]);
    let oub = t.enter(P0, "Oubliette");
    t.resolve_all();
    assert!(t.obj_now(druid).phased_out);
    assert!(!t.obj_now(forest).is(CardType::Creature));
    destroy(&mut t, oub);
    t.g.recompute();
    assert!(!t.obj_now(druid).phased_out);
    assert!(!t.obj_now(forest).is(CardType::Creature));
}

// --- Meandering Towershell ----------------------------------------------------------

/// P0 attacks P1 with the Towershell (exiled), then advances to P0's next declare
/// attackers step, the Towershell returning attacking `defender`.
fn towershell_returns(t: &mut TestGame, ts: ObjectId, defender: PlayerId) {
    attack_with(t, &[(ts, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(ts), Zone::Exile);
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Player(defender)]),
    );
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
}

#[test]
fn meandering_towershell_returns_attacking_a_chosen_player() {
    cr!("508.4", "506.4");
    ruling!(
        "Meandering Towershell",
        "As Meandering Towershell returns to the battlefield because of the delayed triggered ability, you choose which opponent or opposing planeswalker it’s attacking."
    );
    supported("Meandering Towershell");
    let mut t = TestGame::new(3);
    let ts = t.battlefield(P0, "Meandering Towershell");
    towershell_returns(&mut t, ts, P2);
    assert!(t.on_battlefield(ts));
    let c = t.g.combat.as_ref().unwrap();
    let now = t.g.current(ts);
    let attacking: Vec<Entity> = c
        .attackers
        .iter()
        .filter(|a| a.id == now)
        .filter_map(|a| a.target)
        .collect();
    assert_eq!(attacking, vec![Entity::Player(P2)]);
    assert!(t.obj_now(ts).tapped);
}

#[test]
fn meandering_towershell_you_dont_own_returns_under_your_control() {
    cr!("610.3c", "508.4");
    ruling!(
        "Meandering Towershell",
        "If you attack with a Meandering Towershell that you don’t own, you’ll control it when it returns to the battlefield."
    );
    let mut t = TestGame::new(2);
    let ts = t.battlefield(P1, "Meandering Towershell");
    // P0 has controlled it since P0's turn began.
    t.set_step(P1, Step::End);
    crate::r_p015_common::gain_control(&mut t, P0, ts);
    t.advance_to(P0, Step::PrecombatMain);
    towershell_returns(&mut t, ts, P1);
    assert!(t.on_battlefield(ts));
    assert_eq!(t.obj_now(ts).controller, P0);
    assert_eq!(t.obj_now(ts).owner, P1);
}

#[test]
fn meandering_towershell_and_raid() {
    cr!("508.4", "207.2c");
    ruling!(
        "Meandering Towershell",
        "On the turn Meandering Towershell attacks and is exiled, raid abilities will see it as a creature that attacked."
    );
    // Storm Fleet Pyromancer: "Raid — When this creature enters, if you attacked this
    // turn, it deals 2 damage to any target."
    let mut t = TestGame::new(2);
    let ts = t.battlefield(P0, "Meandering Towershell");
    attack_with(&mut t, &[(ts, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Storm Fleet Pyromancer");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // The turn it returns attacking (P0 declared no attackers): no raid.
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Player(P1)]),
    );
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.on_battlefield(ts));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Storm Fleet Pyromancer");
    t.resolve_all();
    assert_eq!(t.life(P1), 13, "only the Towershell's 5 combat damage");
}

// --- Werebear -----------------------------------------------------------------------

#[test]
fn werebear_damage_becomes_lethal_when_threshold_is_lost() {
    cr!("704.5g", "207.2c", "120.6");
    ruling!(
        "Werebear",
        "nonlethal damage dealt to Werebear may become lethal if the number of cards in your graveyard falls below seven."
    );
    supported("Werebear");
    let mut t = TestGame::new(2);
    let wb = t.battlefield(P0, "Werebear");
    let gy: Vec<ObjectId> = (0..7).map(|_| t.graveyard(P0, "Shock")).collect();
    assert_eq!(t.pt(wb), (4, 4));
    crate::r_s06_common::damage(&mut t, wb, 2, wb);
    t.settle();
    assert!(t.on_battlefield(wb));
    t.g.exile_object(gy[0], None);
    t.g.recompute();
    t.settle();
    assert!(in_gy(&t, wb));
}

// --- Phantasmal Dreadmaw -------------------------------------------------------------

#[test]
fn phantasmal_dreadmaw_is_sacrificed_before_the_spell_resolves() {
    cr!("603.3", "608.2b", "117.3");
    ruling!(
        "Phantasmal Dreadmaw",
        "If Phantasmal Dreadmaw becomes the target of a spell or ability, its ability triggers and goes on the stack on top of that spell or ability."
    );
    ruling!(
        "Phantasmal Dreadmaw",
        "Countering the spell or ability will not save Phantasmal Dreadmaw from being sacrificed."
    );
    supported("Phantasmal Dreadmaw");
    // P1's Shock at the Dreadmaw: the trigger is on top; the Dreadmaw is sacrificed; the
    // Shock is removed for having no legal target.
    let mut t = TestGame::new(2);
    let dm = t.battlefield(P0, "Phantasmal Dreadmaw");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let s = t.hand(P1, "Shock");
    let shock = t.cast(P1, s).target(o(dm)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let top = t.g.stack[1];
    assert_eq!(t.obj(top).controller, P0);
    t.resolve();
    assert!(in_gy(&t, dm));
    t.resolve_all();
    assert!(!crate::r_p108_common::resolved(&t, shock));
    // Countering the Shock doesn't stop the trigger.
    let mut t = TestGame::new(2);
    let dm = t.battlefield(P0, "Phantasmal Dreadmaw");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let s = t.hand(P1, "Shock");
    let shock = t.cast(P1, s).target(o(dm)).go();
    t.settle();
    lots(&mut t, P0);
    let cs = t.hand(P0, "Counterspell");
    t.cast(P0, cs).target(o(shock)).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Shock"));
    t.resolve_all();
    assert!(in_gy(&t, dm));
}

// --- Platinum Angel and Hazoret --------------------------------------------------------

#[test]
fn platinum_angel_stops_every_way_of_losing() {
    cr!("104.3b", "104.3c", "704.5a", "704.5b", "704.5c");
    ruling!(
        "Platinum Angel",
        "No game effect can cause you to lose the game or cause any opponent to win the game while you control Platinum Angel."
    );
    supported("Platinum Angel");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Platinum Angel");
    t.g.players[0].life = 0;
    t.g.players[0].counters.insert(counters::POISON.into(), 10);
    t.g.players[0].library.clear();
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(!t.has_lost(P0));
    assert!(t.g.result.is_none());
    destroy(&mut t, angel);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn platinum_angel_protects_the_whole_team_in_two_headed_giant() {
    cr!("810.8a", "104.3b");
    ruling!(
        "Platinum Angel",
        "If you control Platinum Angel in a Two-Headed Giant game, your team can't lose the game and the opposing team can't win the game."
    );
    let mut t = crate::r_p108_common::two_headed_giant();
    t.battlefield(P1, "Platinum Angel");
    for p in [0, 1] {
        t.g.players[p].life = 0;
    }
    t.settle();
    assert!(!t.has_lost(P0));
    assert!(!t.has_lost(P1));
    assert!(t.g.result.is_none());
}

#[test]
fn hazoret_hits_each_opponent_in_two_headed_giant() {
    cr!("810.9a", "120.3a");
    ruling!(
        "Hazoret the Fervent",
        "In a Two-Headed Giant game, Hazoret's activated ability causes 2 damage to be dealt to each opponent, which is 4 damage to the opposing team."
    );
    supported("Hazoret the Fervent");
    let mut t = crate::r_p108_common::two_headed_giant();
    let h = t.battlefield(P0, "Hazoret the Fervent");
    let card = t.hand(P0, "Shock");
    lots(&mut t, P0);
    let (us, them) = (t.life(P0), t.life(P2));
    choose(&mut t, P0, &[card]);
    t.activate(P0, h, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P2), them - 4);
    assert_eq!(t.life(P0), us);
}

#[test]
fn hazoret_stays_in_combat_after_the_hand_grows() {
    cr!("506.4", "508.1c");
    ruling!(
        "Hazoret the Fervent",
        "Once Hazoret has attacked or blocked, it will remain in combat even if the number of cards in your hand becomes two or greater."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hazoret the Fervent");
    t.hand(P0, "Shock");
    attack_with(&mut t, &[(h, Entity::Player(P1))]);
    t.hand(P0, "Shock");
    t.hand(P0, "Shock");
    t.g.recompute();
    assert!(t.g.combat.as_ref().is_some_and(|c| c.attacker(h).is_some()));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}
