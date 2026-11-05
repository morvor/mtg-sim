//! Rulings batch P040 — damage prevention (CR 615) and the cards grouped with it:
//! prevention shields and "prevent all damage" effects, what they cover (noncombat
//! damage, later creatures, redirected combat damage), what they don't (life loss, life
//! payment, the results of damage), combat damage assignment ignoring prevention,
//! "damage that would reduce your life total to less than N", Courageous Resolve's
//! fateful hour, and Sevinne's copies.

use crate::r_p039_creatureland::activate_text;
use crate::r_p040_common::*;
use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, supported, triggers_on_stack,
};
use crate::r_s02_common::create_token;
use crate::r_s04_common::{ability_targets, spell_targets, stack_items};
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s30_common::damage_events;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn set_life(t: &mut TestGame, p: PlayerId, n: i32) {
    t.g.players[p.idx()].life = n;
}

/// P1 casts Sign in Blood targeting `p` ("Target player draws two cards and loses 2
/// life").
fn sign_in_blood(t: &mut TestGame, p: PlayerId) {
    cast_new(t, P1, "Sign in Blood", &[pl(p)]);
    t.resolve_all();
}

/// P1 casts Lava Axe (5 damage) at P0.
fn lava_axe_p0(t: &mut TestGame) {
    cast_new(t, P1, "Lava Axe", &[pl(P0)]);
    t.resolve_all();
}

/// P1's Vampire Nighthawk (2/3 flying, deathtouch, lifelink) made 5/6 by Giant Growth
/// attacks P0 unblocked on P1's turn.
fn lifelink_hits_p0(t: &mut TestGame) {
    t.set_step(P1, Step::PrecombatMain);
    let hawk = old(t, P1, "Vampire Nighthawk");
    cast_new(t, P1, "Giant Growth", &[o(hawk)]);
    t.resolve_all();
    to_combat(t, P1);
    attack_with(t, &[(hawk, pl(P0))]);
    block_and_finish(t, P0, &[]);
    t.resolve_all();
}

/// Damage dealt this turn by `source` to `to`.
fn dealt(t: &TestGame, source: ObjectId, to: Entity) -> u32 {
    damage_events(t)
        .iter()
        .filter(|(s, r, _, _)| *s == source && *r == to)
        .map(|(_, _, n, _)| *n)
        .sum()
}

// ---------------------------------------------------------------------------
// "Damage that would reduce your life total to less than N reduces it to N instead"
// ---------------------------------------------------------------------------

#[test]
fn angels_grace_effects_change_the_result_of_damage_not_prevent_it() {
    cr!("120.3a", "702.15b", "614.1a");
    ruling!(
        "Angel's Grace",
        "Angel's Grace doesn't prevent damage. It only changes the result of damage dealt to you. For example, a 5/5 creature with lifelink that deals damage to you will still cause its controller to gain 5 life, even if that damage reduces your life total from 3 to 1."
    );
    ruling!(
        "Angel of Grace",
        "Angel of Grace's triggered ability doesn't prevent damage. It only changes the result of damage dealt to you. For example, a creature with lifelink that deals damage to you will still cause its controller to gain life, even if that damage would reduce your life total to less than 1."
    );
    ruling!(
        "Elderscale Wurm",
        "Elderscale Wurm’s last ability doesn’t prevent damage. It only changes the result of damage dealt to you. For example, a creature will lifelink that deals damage to you will still cause its controller to gain life, even if that damage would reduce your life total to less than 7."
    );
    // (card, P0's life before, P0's life after)
    for (card, before, after) in [
        ("Angel's Grace", 3, 1),
        ("Angel of Grace", 3, 1),
        ("Elderscale Wurm", 8, 7),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        set_life(&mut t, P0, before);
        match card {
            "Angel's Grace" => {
                cast_new(&mut t, P0, card, &[]);
                t.resolve_all();
            }
            "Angel of Grace" => {
                t.enter(P0, card);
                t.g.flush_events();
                t.resolve_all();
            }
            _ => {
                t.battlefield(P0, card);
            }
        }
        lifelink_hits_p0(&mut t);
        assert_eq!(t.life(P0), after, "{card}");
        assert_eq!(t.life(P1), 25, "{card}");
        assert!(!t.has_lost(P0), "{card}");
    }
}

#[test]
fn life_floors_dont_stop_life_loss() {
    cr!("119.3", "120.3a", "104.3b");
    ruling!(
        "Angel's Grace",
        "Angel's Grace doesn't stop loss of life from effects that say that you lose life."
    );
    ruling!(
        "Angel of Grace",
        "Angel of Grace's triggered ability doesn't stop loss of life from effects that say that you lose life."
    );
    ruling!(
        "Elderscale Wurm",
        "Elderscale Wurm’s last ability doesn’t affect the loss or payment of life."
    );
    ruling!(
        "Fortune Thief",
        "Fortune Thief's effect applies only if your life total is being reduced by damage. Other effects or costs (such as losing or paying life) can reduce your life total below 1 as normal."
    );
    ruling!(
        "Fortune Thief",
        "Fortune Thief won't prevent you from losing the game if your life total becomes 0 or less or if some other effect causes you to lose the game."
    );
    // Angel's Grace: P0 at 1 loses 2 life: -1, but P0 can't lose this turn.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    set_life(&mut t, P0, 1);
    cast_new(&mut t, P0, "Angel's Grace", &[]);
    t.resolve_all();
    sign_in_blood(&mut t, P0);
    assert_eq!(t.life(P0), -1);
    assert!(!t.has_lost(P0));
    // Angel of Grace and Fortune Thief: damage stops at 1, but losing life goes below
    // and P0 loses the game.
    for card in ["Angel of Grace", "Fortune Thief"] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        set_life(&mut t, P0, 3);
        t.enter(P0, card);
        t.g.flush_events();
        t.resolve_all();
        lava_axe_p0(&mut t);
        assert_eq!(t.life(P0), 1, "{card}");
        assert!(!t.has_lost(P0), "{card}");
        sign_in_blood(&mut t, P0);
        assert_eq!(t.life(P0), -1, "{card}");
        assert!(t.has_lost(P0), "{card}");
    }
    // Paying life: Arid Mesa ("{T}, Pay 1 life, Sacrifice this land: ...") at 1 life.
    for card in ["Fortune Thief", "Elderscale Wurm"] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let floor = if card == "Elderscale Wurm" { 7 } else { 1 };
        set_life(&mut t, P0, floor);
        t.battlefield(P0, card);
        let mesa = t.battlefield(P0, "Arid Mesa");
        activate_text(&mut t, P0, mesa, "Pay 1 life").expect("Arid Mesa");
        t.resolve_all();
        assert_eq!(t.life(P0), floor - 1, "{card}");
    }
    // Elderscale Wurm: losing life goes below 7.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    set_life(&mut t, P0, 8);
    t.battlefield(P0, "Elderscale Wurm");
    sign_in_blood(&mut t, P0);
    assert_eq!(t.life(P0), 6);
    // Fortune Thief: another way to lose the game (drawing from an empty library).
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Fortune Thief");
    t.g.players[P0.idx()].library.clear();
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn angel_of_grace_life_total_becomes_ten_by_gaining_or_losing_life() {
    cr!("119.5", "603.2");
    ruling!(
        "Angel of Grace",
        "For a player's life total to become 10, that player gains or loses the appropriate amount of life. For example, if your life total is 2, Angel of Grace's last ability will cause you to gain 8 life; alternatively, if your life total is 20, it will cause you to lose 10 life. Other cards that interact with life gain or life loss will interact with this effect accordingly."
    );
    // (life before, P0's Ajani's Pridemate counters, P1's life after (Exquisite Blood:
    // "Whenever an opponent loses life, you gain that much life"))
    for (before, pridemate, p1_after) in [(2, 1, 20), (20, 0, 30), (10, 0, 20)] {
        let mut t = TestGame::new(2);
        set_life(&mut t, P0, before);
        let pm = t.battlefield(P0, "Ajani's Pridemate");
        t.battlefield(P1, "Exquisite Blood");
        let angel = t.graveyard(P0, "Angel of Grace");
        mana_all(&mut t, P0, 2);
        t.lands(P0, "Plains", 2);
        activate_text(&mut t, P0, angel, "life total becomes 10").expect("Angel of Grace");
        t.resolve_all();
        assert_eq!(t.life(P0), 10, "{before}");
        assert_eq!(t.counters(pm, counters::PLUS1), pridemate, "{before}");
        assert_eq!(t.life(P1), p1_after, "{before}");
        assert!(t.in_exile("Angel of Grace"));
    }
}

// ---------------------------------------------------------------------------
// Courageous Resolve
// ---------------------------------------------------------------------------

/// P0 at `life` casts Courageous Resolve (no target).
fn resolve_at(life: i32) -> TestGame {
    let mut t = TestGame::new(2);
    // On P1's turn, so that P1 can cast sorceries at P0.
    t.set_step(P1, Step::PrecombatMain);
    set_life(&mut t, P0, life);
    t.answer_targets(P0, &[]);
    cast_new(&mut t, P0, "Courageous Resolve", &[]);
    t.resolve_all();
    t
}

#[test]
fn courageous_resolve_checks_life_only_as_it_resolves() {
    cr!("608.2h", "119.8", "104.3c", "104.3d", "104.2");
    ruling!(
        "Courageous Resolve",
        "Courageous Resolve checks your life total only at the time it resolves. If it's 5 or less at that time, even if you gain life later in the turn, you'll still be unable to lose life or lose the game that turn, and your opponents will still be unable to win the game that turn."
    );
    supported("Courageous Resolve");
    let mut t = resolve_at(5);
    // Gaining life later doesn't end it.
    t.g.gain_life(P0, 10);
    t.g.flush_events();
    assert_eq!(t.life(P0), 15);
    lava_axe_p0(&mut t);
    sign_in_blood(&mut t, P0);
    assert_eq!(t.life(P0), 15);
    // Can't lose the game: ten poison counters, an empty library.
    t.g.players[P0.idx()]
        .counters
        .insert(counters::POISON.into(), 10);
    t.g.players[P0.idx()].library.clear();
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(!t.has_lost(P0));
    // Control: at 6 it does nothing.
    let mut t = resolve_at(6);
    lava_axe_p0(&mut t);
    assert_eq!(t.life(P0), 1);
}

#[test]
fn courageous_resolve_stops_opponents_winning() {
    cr!("104.2b", "603.4");
    ruling!(
        "Courageous Resolve",
        "Courageous Resolve checks your life total only at the time it resolves. If it's 5 or less at that time, even if you gain life later in the turn, you'll still be unable to lose life or lose the game that turn, and your opponents will still be unable to win the game that turn."
    );
    // Felidar Sovereign (P1's): "At the beginning of your upkeep, if you have 40 or more
    // life, you win the game." P0 responds to the trigger with Courageous Resolve at 5
    // life, then gains life before the trigger resolves.
    for (life, wins) in [(5, false), (6, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Felidar Sovereign");
        t.g.players[P1.idx()].life = 40;
        set_life(&mut t, P0, life);
        crate::r_s04_common::next_upkeep(&mut t, P1);
        assert_eq!(triggers_on_stack(&t, "you win the game"), 1);
        t.answer_targets(P0, &[]);
        cast_new(&mut t, P0, "Courageous Resolve", &[]);
        t.resolve();
        t.g.gain_life(P0, 10);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.g.player(P1).has_won, wins, "{life}");
        assert!(!t.has_lost(P0) || wins, "{life}");
    }
}

#[test]
fn courageous_resolve_changes_the_results_of_damage_not_prevents_it() {
    cr!("120.3", "702.15b", "702.90b", "603.2");
    ruling!(
        "Courageous Resolve",
        "Courageous Resolve doesn't prevent damage. Rather, it changes the results of that damage. For example, if a creature with lifelink deals damage to you, you won't lose any life, but its controller will still gain that much life. Similarly, if a creature with infect deals damage to you, you'll still receive that many poison counters."
    );
    ruling!(
        "Courageous Resolve",
        "Abilities that trigger whenever damage is dealt to you will still trigger because that damage is still dealt, even though your life total won't change as a result."
    );
    // Lifelink.
    let mut t = resolve_at(5);
    lifelink_hits_p0(&mut t);
    assert_eq!(t.life(P0), 5);
    assert_eq!(t.life(P1), 25);
    // Infect: Plague Stinger (1/1 flying, infect).
    let mut t = resolve_at(5);
    let stinger = old(&mut t, P1, "Plague Stinger");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(stinger, pl(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.g.player(P0).poison(), 1);
    assert_eq!(t.life(P0), 5);
    // Sun Droplet: "Whenever you're dealt damage, put that many charge counters on Sun
    // Droplet."
    let mut t = resolve_at(5);
    let droplet = t.battlefield(P0, "Sun Droplet");
    cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 5);
    assert_eq!(t.counters(droplet, "charge"), 2);
}

#[test]
fn courageous_resolve_turns_life_loss_replacements_into_nothing() {
    cr!("614.1a", "119.8");
    ruling!(
        "Courageous Resolve",
        "Effects that replace an event with having you lose life (such as Tainted Remedy) will end up replacing the event with nothing."
    );
    // Tainted Remedy (P1's): "If an opponent would gain life, that player loses that much
    // life instead."
    let mut t = resolve_at(5);
    t.battlefield(P1, "Tainted Remedy");
    cast_new(&mut t, P0, "Healing Salve", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 5);
    // Control: without the fateful hour effect P0 loses the 3.
    let mut t = resolve_at(6);
    t.battlefield(P1, "Tainted Remedy");
    cast_new(&mut t, P0, "Healing Salve", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 3);
}

// ---------------------------------------------------------------------------
// What "prevent all damage" covers
// ---------------------------------------------------------------------------

#[test]
fn prevention_of_all_damage_covers_noncombat_damage() {
    cr!("615.1a", "120.2b");
    ruling!(
        "Blinding Fog",
        "Blinding Fog prevents all damage that would be dealt to creatures, not just combat damage."
    );
    ruling!(
        "Emmara Tandris",
        "Emmara Tandris’s ability prevents all damage that would be dealt to creature tokens you control, not just combat damage."
    );
    supported("Blinding Fog");
    supported("Emmara Tandris");
    // Blinding Fog: Shock on P1's creature, then combat damage between creatures.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let theirs = old(&mut t, P1, "Hill Giant");
    let mine = old(&mut t, P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Blinding Fog", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Shock", &[o(theirs)]);
    t.resolve_all();
    assert_eq!(damage(&t, theirs), 0);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(mine, pl(P1))]);
    block_and_finish(&mut t, P1, &[(theirs, mine)]);
    assert!(t.on_battlefield(mine));
    assert_eq!((damage(&t, theirs), damage(&t, mine)), (0, 0));
    // Players are still dealt damage.
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Emmara Tandris: Shock on a creature token P0 controls; control: a nontoken one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Emmara Tandris");
    let tok = create_token(&mut t, P0, "Soldier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = create_token(&mut t, P1, "Soldier");
    for target in [tok, bears, theirs] {
        cast_new(&mut t, P1, "Shock", &[o(target)]);
        t.resolve_all();
    }
    assert!(t.on_battlefield(tok));
    assert_eq!(damage(&t, tok), 0);
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn tresserhorn_skyknight_prevents_all_damage_from_first_strikers() {
    cr!("615.1a", "702.7b", "702.4b", "701.14a");
    ruling!(
        "Tresserhorn Skyknight",
        "All damage (not just combat damage and not just first-strike combat damage) that would be dealt to Tresserhorn Skyknight by creatures with first strike is prevented."
    );
    ruling!(
        "Tresserhorn Skyknight",
        "Damage that would be dealt to Tresserhorn Skyknight by creatures with double strike is dealt as normal."
    );
    supported("Tresserhorn Skyknight");
    // Noncombat: P1's Boros Recruit (1/1 first strike) fights it (Prey Upon).
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let sky = old(&mut t, P0, "Tresserhorn Skyknight");
    let recruit = old(&mut t, P1, "Boros Recruit");
    cast_new(&mut t, P1, "Prey Upon", &[o(recruit), o(sky)]);
    t.resolve_all();
    assert_eq!(dealt(&t, recruit, o(sky)), 0);
    assert_eq!(damage(&t, sky), 0);
    assert!(!t.on_battlefield(recruit));
    // Combat: it blocks a first striker: no damage. A double striker: 1 + 1.
    for (attacker, want) in [("Boros Recruit", 0), ("Fencing Ace", 2)] {
        let mut t = TestGame::new(2);
        let sky = old(&mut t, P0, "Tresserhorn Skyknight");
        let a = old(&mut t, P1, attacker);
        t.set_step(P1, Step::BeginningOfCombat);
        attack_with(&mut t, &[(a, pl(P0))]);
        block_and_finish(&mut t, P0, &[(sky, a)]);
        assert_eq!(damage(&t, sky), want, "{attacker}");
        assert_eq!(dealt(&t, a, o(sky)), want, "{attacker}");
    }
}

#[test]
fn magebane_armor_doesnt_prevent_redirected_combat_damage() {
    cr!("510.2", "615.1a", "120.2a");
    ruling!(
        "Magebane Armor",
        "Combat damage is the damage that’s dealt automatically by attacking and blocking creatures, even if it’s redirected (by Harm’s Way, perhaps). Noncombat damage is any other damage (generally, it’s damage dealt as the result of a spell or ability)."
    );
    supported("Magebane Armor");
    supported("Harm's Way");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Magebane Armor", o(bears));
    assert_eq!(t.pt(bears), (4, 6));
    // Noncombat damage to it is prevented.
    cast_new(&mut t, P1, "Shock", &[o(bears)]);
    t.resolve_all();
    assert_eq!(damage(&t, bears), 0);
    // P1's Hill Giant attacks P0; P0 redirects 2 of its damage to the Bears.
    let giant = old(&mut t, P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, pl(P0))]);
    t.answer_choose(P0, &[o(giant)]);
    cast_new(&mut t, P0, "Harm's Way", &[o(bears)]);
    t.resolve_all();
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(damage(&t, bears), 2);
    assert_eq!(t.life(P0), 19);
    assert!(damage_events(&t)
        .iter()
        .any(|(s, r, n, combat)| *s == giant && *r == o(bears) && *n == 2 && *combat));
}

#[test]
fn blessed_sanctuary_prevents_noncombat_damage_even_during_combat_but_not_life_loss() {
    cr!("120.2b", "510.2", "615.1a", "119.3");
    ruling!(
        "Blessed Sanctuary",
        "Combat damage is the damage that’s dealt automatically by attacking and blocking creatures. Any other damage is noncombat damage, even if it’s dealt during a combat phase by an attacking or blocking creature."
    );
    ruling!(
        "Blessed Sanctuary",
        "Effects that cause you to lose life aren’t prevented."
    );
    supported("Blessed Sanctuary");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blessed Sanctuary");
    let bears = old(&mut t, P0, "Grizzly Bears");
    let giant = old(&mut t, P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, pl(P0))]);
    // Soul's Fire: the attacking Giant deals 3 damage to P0's Bears: noncombat,
    // prevented.
    cast_new(&mut t, P1, "Soul's Fire", &[o(giant), o(bears)]);
    t.resolve_all();
    assert_eq!(damage(&t, bears), 0);
    assert!(t.on_battlefield(bears));
    // Its combat damage to P0 isn't prevented.
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 17);
    // Losing life isn't prevented.
    t.set_step(P1, Step::PostcombatMain);
    sign_in_blood(&mut t, P0);
    assert_eq!(t.life(P0), 15);
}

#[test]
fn gloom_surgeon_prevents_even_when_the_library_is_short() {
    cr!("615.1a", "614.1a");
    ruling!(
        "Gloom Surgeon",
        "Combat damage that would be dealt to Gloom Surgeon will be prevented even if you can’t exile that many cards from the top of your library."
    );
    supported("Gloom Surgeon");
    let mut t = TestGame::new(2);
    let surgeon = old(&mut t, P0, "Gloom Surgeon");
    t.g.players[P0.idx()].library.clear();
    t.library_top(P0, "Island");
    let giant = old(&mut t, P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, pl(P0))]);
    block_and_finish(&mut t, P0, &[(surgeon, giant)]);
    assert!(t.on_battlefield(surgeon));
    assert_eq!(damage(&t, surgeon), 0);
    assert_eq!(t.library_size(P0), 0);
    assert!(t.in_exile("Island"));
    assert_eq!(damage(&t, giant), 2);
}

#[test]
fn blinding_fog_covers_later_creatures_but_hexproof_only_current_ones() {
    cr!("611.2c", "615.1a", "702.11b");
    ruling!(
        "Blinding Fog",
        "Damage that would be dealt to creatures that enter the battlefield later in the turn will also be prevented. However, creatures that come under your control later in the turn won’t gain hexproof."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = old(&mut t, P0, "Grizzly Bears");
    let stolen = old(&mut t, P1, "Hill Giant");
    cast_new(&mut t, P0, "Blinding Fog", &[]);
    t.resolve_all();
    // A creature entering later: its damage is prevented.
    let later = t.enter(P1, "Grizzly Bears");
    t.g.flush_events();
    cast_new(&mut t, P0, "Shock", &[o(later)]);
    t.resolve_all();
    assert!(t.on_battlefield(later));
    assert_eq!(damage(&t, later), 0);
    // A creature P0 gains control of later doesn't have hexproof; P0's Bears do.
    cast_new(&mut t, P0, "Act of Treason", &[o(stolen)]);
    t.resolve_all();
    assert_eq!(t.obj_now(stolen).controller, P0);
    let targets = spell_targets(&mut t, P1, "Shock");
    assert!(targets.contains(&o(stolen)));
    assert!(!targets.contains(&o(mine)));
}

// ---------------------------------------------------------------------------
// Combat damage assignment ignores prevention
// ---------------------------------------------------------------------------

#[test]
fn prevention_doesnt_change_lethal_damage_for_trample() {
    cr!("702.19b", "510.1c", "615.1a");
    ruling!(
        "Everdawn Champion",
        "Combat damage can be assigned to Everdawn Champion as normal, even though that damage will be prevented. For example, if Everdawn Champion blocks a 4/4 creature with trample, the attacking player may assign 2 of that creature’s combat damage to the player or planeswalker it’s attacking."
    );
    ruling!(
        "Fog Bank",
        "Fog Bank's prevention effect isn't considered while assigning combat damage from a creature with trample. For example, if it blocks a 5/5 creature with trample, that creature's controller must assign 2 of that creature's combat damage to Fog Bank and the remainder can be assigned to the defending player, planeswalker, or battle."
    );
    for card in ["Everdawn Champion", "Fog Bank"] {
        supported(card);
        // A 5/5 trampler (War Mammoth with +2/+2): 2 to the blocker, 3 to P0.
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let blocker = old(&mut t, P0, card);
        let mammoth = old(&mut t, P1, "War Mammoth");
        t.battlefield(P1, "Glorious Anthem");
        t.battlefield(P1, "Glorious Anthem");
        assert_eq!(t.pt(mammoth), (5, 5));
        to_combat(&mut t, P1);
        attack_with(&mut t, &[(mammoth, pl(P0))]);
        let from = mark(&t);
        t.answer(P1, DecisionKind::Damage, Answer::Numbers(vec![2, 3]));
        block_and_finish(&mut t, P0, &[(blocker, mammoth)]);
        assert_eq!(lethal_offers(&t, from), vec![vec![2]], "{card}");
        assert_eq!(t.life(P0), 17, "{card}");
        assert_eq!(damage(&t, blocker), 0, "{card}");
        assert!(t.on_battlefield(blocker), "{card}");
    }
}

// ---------------------------------------------------------------------------
// Prevention shields and choices
// ---------------------------------------------------------------------------

#[test]
fn rakalite_can_be_activated_any_number_of_times() {
    cr!("602.2", "615.7");
    ruling!(
        "Rakalite",
        "Can be used any number of times during the turn before it leaves the battlefield."
    );
    supported("Rakalite");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let r = t.battlefield(P0, "Rakalite");
    t.g.players[P0.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::C, 6);
    for _ in 0..3 {
        t.answer_targets(P0, &[pl(P0)]);
        activate_text(&mut t, P0, r, "Prevent the next 1").expect("Rakalite");
        t.resolve_all();
    }
    lava_axe_p0(&mut t);
    assert_eq!(t.life(P0), 18);
    // It returns to its owner's hand at the beginning of the next end step.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Rakalite"));
}

#[test]
fn consulate_surveillance_chooses_its_source_on_resolution_without_targeting() {
    cr!("609.7a", "115.10a", "702.18a");
    ruling!(
        "Consulate Surveillance",
        "Consulate Surveillance's second ability doesn't target anything. You choose a source of damage as it resolves."
    );
    supported("Consulate Surveillance");
    let mut t = TestGame::new(2);
    let cs = t.enter(P0, "Consulate Surveillance");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 4);
    // Neurok Commando has shroud: it can't be targeted, but it can be chosen.
    let commando = old(&mut t, P1, "Neurok Commando");
    t.answer_choose(P0, &[o(commando)]);
    let from = mark(&t);
    activate_text(&mut t, P0, cs, "Prevent all damage").expect("Consulate Surveillance");
    let targets_asked = asked_since(&t, from)
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count();
    assert_eq!(targets_asked, 0);
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 2);
    t.resolve_all();
    let chose = asked_since(&t, from)
        .into_iter()
        .any(|(p, d)| p == P0 && matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.contains(&o(commando))));
    assert!(chose);
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(commando, pl(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn atalyas_x_needs_white_mana_but_an_added_cost_doesnt() {
    cr!("107.3", "601.2f", "602.2b");
    ruling!(
        "Atalya, Samite Master",
        "Cost modifiers which add a cost are not restricted to only white mana."
    );
    supported("Atalya, Samite Master");
    supported("Suppression Field");
    // Suppression Field: "Activated abilities cost {2} more to activate unless they're
    // mana abilities." Atalya with X = 1, gaining life: {W} for X, {2} from Islands.
    let mut t = TestGame::new(2);
    let a = old(&mut t, P0, "Atalya, Samite Master");
    t.battlefield(P1, "Suppression Field");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    activate_text(&mut t, P0, a, "Choose one").expect("Atalya");
    // All three lands paid: {W} for X and {2} for Suppression Field.
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Control: X itself can't be paid with blue mana.
    let mut t = TestGame::new(2);
    let a = old(&mut t, P0, "Atalya, Samite Master");
    t.battlefield(P1, "Suppression Field");
    t.lands(P0, "Island", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    assert!(activate_text(&mut t, P0, a, "Choose one").is_err());
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 3);
}

#[test]
fn cover_of_winter_prevents_combat_damage_to_you_and_your_combatants() {
    cr!("615.1a", "510.2");
    ruling!(
        "Cover of Winter",
        "Cover of Winter's replacement effect prevents combat damage creatures would deal to you and to attacking or blocking creatures you control."
    );
    supported("Cover of Winter");
    // One age counter: prevent 1 of each.
    let mut t = TestGame::new(2);
    let cover = t.battlefield(P0, "Cover of Winter");
    t.g.add_counters(o(cover), "age", 1, None);
    t.g.flush_events();
    let wall = old(&mut t, P0, "Wall of Stone");
    let giant = old(&mut t, P1, "Hill Giant");
    let bears = old(&mut t, P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, pl(P0)), (bears, pl(P0))]);
    block_and_finish(&mut t, P0, &[(wall, bears)]);
    // To P0: 3 - 1; to the blocking Wall: 2 - 1.
    assert_eq!(t.life(P0), 18);
    assert_eq!(damage(&t, wall), 1);
    // P0's attacking creature: the Giant (3/3) blocked by P1's Bears takes 2 - 1.
    let mut t = TestGame::new(2);
    let cover = t.battlefield(P0, "Cover of Winter");
    t.g.add_counters(o(cover), "age", 1, None);
    t.g.flush_events();
    let giant = old(&mut t, P0, "Hill Giant");
    let bears = old(&mut t, P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(giant, pl(P1))]);
    block_and_finish(&mut t, P1, &[(bears, giant)]);
    assert_eq!(damage(&t, giant), 1);
    // Noncombat damage isn't prevented.
    cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn ajani_steadfast_emblem_prevents_per_source() {
    cr!("615.1a", "114.4");
    ruling!(
        "Ajani Steadfast",
        "Each attacking creature is a separate source of damage. For example, if two 3/3 creatures attack you and are unblocked, the emblem will prevent 2 damage from each creature. You’ll be dealt a total of 2 damage."
    );
    supported("Ajani Steadfast");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Steadfast");
    t.g.objects[ajani.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    loyalty(&mut t, P0, ajani, "−7:", &[]);
    let a = old(&mut t, P1, "Hill Giant");
    let b = old(&mut t, P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a, pl(P0)), (b, pl(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn indestructible_aura_prevents_damage_only_and_isnt_an_aura() {
    cr!("615.1a", "701.8a", "303.4");
    ruling!(
        "Indestructible Aura",
        "Despite the name, this card only prevents damage and not destroy effects. It’s also not an Aura."
    );
    supported("Indestructible Aura");
    let c = mtg_engine::card::card("Indestructible Aura");
    assert!(c.front().chars.card_types.contains(CardType::Instant));
    assert!(!c.front().chars.has_subtype("Aura"));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Indestructible Aura", &[o(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Indestructible Aura"));
    cast_new(&mut t, P1, "Lightning Bolt", &[o(bears)]);
    t.resolve_all();
    assert_eq!(damage(&t, bears), 0);
    cast_new(&mut t, P1, "Doom Blade", &[o(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn djerus_resolve_can_target_an_untapped_creature() {
    cr!("115.1a", "608.2b", "615.1a");
    ruling!(
        "Djeru's Resolve",
        "Djeru’s Resolve can target an untapped creature. Its prevention effect will still be applied."
    );
    supported("Djeru's Resolve");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!t.obj_now(bears).tapped);
    assert!(spell_targets(&mut t, P0, "Djeru's Resolve").contains(&o(bears)));
    cast_new(&mut t, P0, "Djeru's Resolve", &[o(bears)]);
    t.resolve_all();
    cast_new(&mut t, P1, "Lightning Bolt", &[o(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(damage(&t, bears), 0);
}

#[test]
fn dovin_can_target_any_permanent_an_opponent_controls() {
    cr!("115.1");
    ruling!(
        "Dovin, Hand of Control",
        "Dovin’s loyalty ability can target any permanent an opponent controls, not just one that can deal or be dealt damage."
    );
    supported("Dovin, Hand of Control");
    let mut t = TestGame::new(2);
    let dovin = t.battlefield(P0, "Dovin, Hand of Control");
    let forest = t.battlefield(P1, "Forest");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Glorious Anthem");
    let cands = ability_targets(&mut t, dovin, 0);
    for x in [forest, anthem, bears] {
        assert!(cands.contains(&o(x)));
    }
    assert!(!cands.contains(&o(mine)));
    loyalty(&mut t, P0, dovin, "−1:", &[o(forest)]);
    assert_eq!(t.counters(dovin, "loyalty"), 4);
}

#[test]
fn energy_field_is_sacrificed_by_discards_but_not_by_buyback_spells() {
    cr!("702.27a", "701.9a", "603.2");
    ruling!(
        "Energy Field",
        "Buyback causes a spell to go to your hand instead of your graveyard, so using Buyback will avoid triggering this card to be sacrificed."
    );
    ruling!("Energy Field", "Discarding does trigger this.");
    supported("Energy Field");
    // Buyback paid: the spell returns to hand, the Field stays.
    for buyback in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Energy Field");
        t.lands(P0, "Island", 6);
        let w = t.hand(P0, "Whispers of the Muse");
        t.cast(P0, w).kicked(buyback).go();
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Whispers of the Muse"), buyback);
        assert_eq!(t.in_graveyard(P0, "Energy Field"), !buyback, "{buyback}");
    }
    // Discarding.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Energy Field");
    let c = t.hand(P0, "Island");
    t.g.discard(P0, c, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Energy Field"));
}

#[test]
fn archon_of_coronation_still_counts_commander_damage() {
    cr!("903.10a", "120.3a");
    ruling!(
        "Archon of Coronation",
        "Combat damage dealt to you by a commander is still tracked, even if it didn't cause you to lose life."
    );
    let mut t = crate::r_s13_common::commander_game();
    t.battlefield(P0, "Archon of Coronation");
    t.g.monarch = Some(P0);
    let giant = old(&mut t, P1, "Hill Giant");
    t.g.objects[giant.0 as usize].is_commander = true;
    t.g.players[P1.idx()]
        .commander_names
        .push("Hill Giant".into());
    t.g.recompute();
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, pl(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 40);
    assert_eq!(
        t.g.player(P0).commander_damage.get("Hill Giant").copied(),
        Some(3)
    );
}

// ---------------------------------------------------------------------------
// Sevinne
// ---------------------------------------------------------------------------

#[test]
fn sevinne_copies_even_a_countered_spell_and_the_copy_resolves_first() {
    cr!("707.10", "405.2", "702.34a");
    ruling!(
        "Sevinne, the Chronoclasm",
        "A copy is created even if the spell that caused Sevinne's ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell."
    );
    supported("Sevinne, the Chronoclasm");
    let flashback =
        mtg_engine::object::CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback);
    // The original is countered: the copy still draws.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Sevinne, the Chronoclasm");
    let tt = t.graveyard(P0, "Think Twice");
    mana_all(&mut t, P0, 1);
    let spell = t.cast(P0, tt).method(flashback.clone()).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "copy that spell"), 1);
    cast_new(&mut t, P1, "Counterspell", &[o(spell)]);
    t.resolve();
    t.resolve();
    assert!(t.in_exile("Think Twice"));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Not countered: the copy is put on the stack above the original.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Sevinne, the Chronoclasm");
    let tt = t.graveyard(P0, "Think Twice");
    mana_all(&mut t, P0, 1);
    let spell = t.cast(P0, tt).method(flashback).go();
    t.resolve();
    assert_eq!(t.g.stack.len(), 2);
    assert_eq!(t.g.stack[0], spell);
    let copy = t.g.stack[1];
    assert_eq!(t.g.obj(copy).kind, mtg_engine::object::ObjKind::SpellCopy);
    assert_eq!(stack_items(&t), vec!["Think Twice", "Think Twice"]);
    t.resolve();
    assert!(!t.g.stack.contains(&copy));
    assert!(t.g.stack.contains(&spell));
}
