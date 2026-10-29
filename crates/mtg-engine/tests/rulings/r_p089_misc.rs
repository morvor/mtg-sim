//! Rulings batch P089 — Martial Coup, homeward effects (Crack in Time, Brooding Saurian,
//! Flicker), improvise cards, and impulse effects (Expressive Iteration, Sea Gate Oracle,
//! Stress Dream, Sword-Point Diplomacy).

use crate::r_p089_common::*;
use crate::r_s05_common::{enter, move_to};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    crate::r_p076_common::mana(t, p, ty, n);
}

// ---------------------------------------------------------------------------------------
// Martial Coup
// ---------------------------------------------------------------------------------------

#[test]
fn martial_coup_creates_tokens_then_destroys_the_rest() {
    cr!("608.2c", "117.2e");
    ruling!(
        "Martial Coup",
        "You do what the spell says in order. If X is 5 or more, you'll put the Soldier tokens onto the battlefield, then you'll destroy all other creatures."
    );
    ruling!(
        "Martial Coup",
        "No one can cast spells or activate abilities between the time the Soldier tokens are put onto the battlefield and the time all other creatures are destroyed."
    );
    supported("Martial Coup");
    supported("Skeletal Kathari");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let kathari = t.battlefield(P1, "Skeletal Kathari");
    let theirs = t.battlefield(P1, "Hill Giant");
    let coup = t.hand(P0, "Martial Coup");
    pool(&mut t, P0, ManaType::W, 7);
    t.cast(P0, coup).x(5).go();
    t.settle();
    let from = t.asked().len();
    t.g.resolve_top();
    // Nobody got priority while it resolved.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    t.settle();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 5);
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(kathari));
    assert!(!t.on_battlefield(theirs));
    // With X = 4: only the tokens.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let coup = t.hand(P0, "Martial Coup");
    pool(&mut t, P0, ManaType::W, 6);
    t.cast(P0, coup).x(4).go();
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 4);
    assert!(t.on_battlefield(mine));
}

// ---------------------------------------------------------------------------------------
// Homeward effects
// ---------------------------------------------------------------------------------------

#[test]
fn crack_in_time_gone_before_its_trigger_resolves() {
    cr!("610.3b");
    ruling!(
        "Crack in Time",
        "If Crack in Time leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    supported("Crack in Time");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let crack = enter(&mut t, P0, "Crack in Time");
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, crack, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Control: with Crack in Time still there, it's exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Crack in Time");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn brooding_saurian_overrides_confiscate() {
    cr!("613.1b", "613.7");
    ruling!(
        "Brooding Saurian",
        "If a permanent is enchanted with an Aura such as Confiscate when Brooding Saurian’s ability resolves, the enchanted permanent’s owner will regain control of it. The Aura will not be removed."
    );
    supported("Brooding Saurian");
    supported("Confiscate");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Brooding Saurian");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let conf = attach_new(&mut t, P0, "Confiscate", obj(bears));
    assert_eq!(t.obj_now(bears).controller, P0);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    assert!(t.on_battlefield(conf));
    assert_eq!(t.obj_now(conf).attached_to, Some(obj(bears)));
    assert_eq!(t.obj_now(conf).controller, P0);
}

#[test]
fn flicker_triggers_enters_and_leaves_but_not_dies() {
    cr!("603.6a", "603.6c", "700.4", "400.7");
    ruling!(
        "Flicker",
        "It will trigger \"enters\" abilities and \"leaves the battlefield\" abilities. Note that the card does not go to the graveyard"
    );
    supported("Flicker");
    supported("Aven Riftwatcher");
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, "Aven Riftwatcher");
    t.battlefield(P0, "Blood Artist");
    cast_and_resolve(&mut t, P0, "Flicker", &[obj(rift)]);
    assert!(t.on_battlefield(rift));
    // Leaves (2) and enters (2); Blood Artist didn't trigger.
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn flicker_resets_the_permanent() {
    cr!("400.7", "704.5m", "122.2");
    ruling!(
        "Flicker",
        "All counters, Auras, effects, and so on are removed when it is exiled. Then it comes back onto the battlefield like new."
    );
    supported("Flicker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 2);
    attach_new(&mut t, P0, "Holy Strength", obj(bears));
    cast_and_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.pt(bears), (8, 9));
    cast_and_resolve(&mut t, P0, "Flicker", &[obj(bears)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert!(t.in_graveyard(P0, "Holy Strength"));
}

#[test]
fn flickered_permanent_is_a_new_object_for_other_spells() {
    cr!("400.7", "608.2b");
    ruling!(
        "Flicker",
        "This could be used to make the target of another spell or ability illegal if this wasn't a sorcery"
    );
    supported("Flicker");
    supported("Leyline of Anticipation");
    let mut t = TestGame::new(2);
    // Leyline of Anticipation: P0 may cast Flicker as though it had flash.
    t.battlefield(P0, "Leyline of Anticipation");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shock = cast_new(&mut t, P1, "Shock", &[obj(bears)]);
    cast_and_resolve(&mut t, P0, "Flicker", &[obj(bears)]);
    assert!(!t.g.stack.contains(&shock));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

// ---------------------------------------------------------------------------------------
// Improvise cards
// ---------------------------------------------------------------------------------------

#[test]
fn battle_at_the_bridge_illegal_target_no_life() {
    cr!("608.2b");
    ruling!(
        "Battle at the Bridge",
        "If the target creature becomes an illegal target, Battle at the Bridge doesn’t resolve and none of its effects happen. You won’t gain life."
    );
    supported("Battle at the Bridge");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Battle at the Bridge");
    pool(&mut t, P0, ManaType::B, 3);
    t.cast(P0, spell).x(2).target(bears).go();
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Control: with the target there, P0 gains 2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Battle at the Bridge");
    pool(&mut t, P0, ManaType::B, 3);
    t.cast(P0, spell).x(2).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(bears));
}

/// Whether `id` is attacking after P0 declares no attackers in a fresh combat.
fn attacks_when_declining(t: &mut TestGame, id: ObjectId) -> bool {
    crate::r_s20_common::to_beginning_of_combat(t, P0);
    attack_with(t, &[]);
    t.g.is_attacking(t.g.current(id))
}

#[test]
fn barricade_breaker_attacks_if_able_without_paying_costs() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Barricade Breaker",
        "If there's a cost associated with having it attack, you aren't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Barricade Breaker");
    supported("Ghostly Prison");
    // Untapped and unrestricted: it attacks.
    let mut t = TestGame::new(2);
    let bb = t.battlefield(P0, "Barricade Breaker");
    assert!(attacks_when_declining(&mut t, bb));
    // Tapped: it doesn't.
    let mut t = TestGame::new(2);
    let bb = t.battlefield(P0, "Barricade Breaker");
    t.g.objects[bb.0 as usize].tapped = true;
    assert!(!attacks_when_declining(&mut t, bb));
    // A cost to attack (Ghostly Prison), with mana available: it needn't attack.
    let mut t = TestGame::new(2);
    let bb = t.battlefield(P0, "Barricade Breaker");
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Plains", 2);
    assert!(!attacks_when_declining(&mut t, bb));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
}

// ---------------------------------------------------------------------------------------
// Impulse
// ---------------------------------------------------------------------------------------

/// Empties `p`'s library and puts `top_first` in it (the first on top).
fn library(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    t.g.players[p.idx()].library.clear();
    crate::r_s01_common::stack_library(t, p, top_first)
}

#[test]
fn expressive_iteration_with_two_cards() {
    cr!("609.3");
    ruling!(
        "Expressive Iteration",
        "If there are fewer than three cards in your library, follow the instructions in the order given for any cards that are there."
    );
    supported("Expressive Iteration");
    let mut t = TestGame::new(2);
    let cards = library(&mut t, P0, &["Grizzly Bears", "Shock"]);
    t.answer_choose(P0, &[obj(cards[1])]);
    cast_and_resolve(&mut t, P0, "Expressive Iteration", &[]);
    assert_eq!(t.zone(cards[1]), Zone::Hand(P0));
    assert_eq!(t.zone(cards[0]), Zone::Library(P0));
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn sea_gate_oracle_with_one_card() {
    cr!("609.3");
    ruling!(
        "Sea Gate Oracle",
        "If there’s only one card in your library as Sea Gate Oracle’s ability resolves, you’ll put it into your hand."
    );
    supported("Sea Gate Oracle");
    let mut t = TestGame::new(2);
    let cards = library(&mut t, P0, &["Shock"]);
    enter(&mut t, P0, "Sea Gate Oracle");
    t.resolve_all();
    assert_eq!(t.zone(cards[0]), Zone::Hand(P0));
}

/// P0 resolves Expressive Iteration with `top_first` on top of their library, putting the
/// first into their hand and the second on the bottom (the third is exiled). Returns the
/// cards.
fn iterate(t: &mut TestGame, top_first: &[&str; 3]) -> Vec<ObjectId> {
    let cards = library(t, P0, &top_first[..]);
    // Cards to draw later, under them.
    for _ in 0..10 {
        let w = t.library_top(P0, "Wastes");
        let lib = &mut t.g.players[0].library;
        lib.retain(|c| *c != w);
        lib.insert(0, w);
    }
    t.answer_choose(P0, &[obj(cards[0])]);
    t.answer_choose(P0, &[obj(cards[1])]);
    cast_and_resolve(t, P0, "Expressive Iteration", &[]);
    assert_eq!(t.zone(cards[2]), Zone::Exile);
    cards
}

#[test]
fn expressive_iteration_exiled_card_follows_normal_rules() {
    cr!("305.2", "601.2f", "611.2a");
    ruling!(
        "Expressive Iteration",
        "If you choose to play the exiled card this turn, you must still pay all costs and follow all timing restrictions required by that card. If it's a land, you can't play it unless you have a land play available."
    );
    ruling!(
        "Expressive Iteration",
        "If you don't play the exiled card, it remains exiled. It won't be available to be played on future turns."
    );
    supported("Expressive Iteration");
    // A land: only with a land play available.
    let mut t = TestGame::new(2);
    let cards = iterate(&mut t, &["Grizzly Bears", "Shock", "Mountain"]);
    let land = t.g.current(cards[2]);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    assert!(t.play_land(P0, land).is_err());
    let mut t = TestGame::new(2);
    let cards = iterate(&mut t, &["Grizzly Bears", "Shock", "Mountain"]);
    let land = t.g.current(cards[2]);
    assert!(t.play_land(P0, land).is_ok());
    // A spell: its mana cost must be paid.
    let mut t = TestGame::new(2);
    let cards = iterate(&mut t, &["Grizzly Bears", "Mountain", "Shock"]);
    let shock = t.g.current(cards[2]);
    assert!(t.cast(P0, shock).target(player(P1)).try_go().is_err());
    t.clear_answers();
    pool(&mut t, P0, ManaType::R, 1);
    assert!(t.cast(P0, shock).target(player(P1)).try_go().is_ok());
    // Not played this turn: it stays exiled and can't be played on P0's next turn.
    let mut t = TestGame::new(2);
    let cards = iterate(&mut t, &["Grizzly Bears", "Shock", "Mountain"]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let land = t.g.current(cards[2]);
    assert_eq!(t.zone(land), Zone::Exile);
    assert!(t.play_land(P0, land).is_err());
}

#[test]
fn stress_dream_target_is_optional_but_checked() {
    cr!("115.1", "608.2b", "601.2c");
    ruling!(
        "Stress Dream",
        "You don't have to choose a target for Stress Dream. However, if you do and the target creature is an illegal target as Stress Dream tries to resolve, it won't resolve and none of its effects will happen. You won't look at any cards in your library."
    );
    supported("Stress Dream");
    // No target: P0 still gets a card.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[]);
    cast_and_resolve(&mut t, P0, "Stress Dream", &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    // A target that becomes illegal: nothing happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    let lib = t.library_size(P0);
    cast_new(&mut t, P0, "Stress Dream", &[obj(bears)]);
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), lib);
    assert!(!t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. })));
}

#[test]
fn sword_point_diplomacy_reveals_all_before_paying() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Sword-Point Diplomacy",
        "You reveal all three cards before opponents choose whether to pay life for any of them."
    );
    supported("Sword-Point Diplomacy");
    let mut t = TestGame::new(2);
    library(&mut t, P0, &["Grizzly Bears", "Shock", "Hill Giant", "Forest"]);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| !matches!(d, Decision::Priority { .. }),
        |g| g.reveals.revealed.len(),
    );
    // P1 pays for the first card only.
    t.answer(P1, DecisionKind::Any, Answer::Bool(true));
    t.answer(P1, DecisionKind::Any, Answer::Bool(false));
    t.answer(P1, DecisionKind::Any, Answer::Bool(false));
    cast_and_resolve(&mut t, P0, "Sword-Point Diplomacy", &[]);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 3);
    assert!(seen.iter().all(|n| *n >= 3), "{seen:?}");
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_hand(P0, "Shock") && t.in_hand(P0, "Hill Giant"));
}
