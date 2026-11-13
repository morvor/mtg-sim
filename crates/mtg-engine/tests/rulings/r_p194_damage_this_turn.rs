//! Rulings batch P194 — abilities that count the damage dealt, cards drawn, tokens
//! created or cards put into a graveyard this turn: what counts (prevented damage
//! doesn't, damage that caused no life loss does; loss of life isn't damage), and the
//! amount read as the ability resolves (CR 608.2h).

use crate::r_p194_common::*;
use crate::r_s01_common::{supported, tokens, with_subtype};
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Shieldmate's Blessing ("Prevent the next 3 damage that would be dealt to any
/// target this turn") on `target`.
fn shield(t: &mut TestGame, target: Entity) {
    let s = t.hand(P0, "Shieldmate's Blessing");
    mana(t, P0, ManaType::W, 1);
    t.cast(P0, s).target(target).go();
    t.resolve_all();
}

#[test]
fn niko_deals_its_damage_at_once() {
    cr!("120.2b", "608.2h");
    ruling!(
        "Niko Aris",
        "Niko deals all the damage for their second ability at the same time. For example, if you've drawn two cards this turn, Niko will deal 4 damage to the target tapped creature; they won't deal 2 damage, then deal 2 more damage."
    );
    supported("Niko Aris");
    let mut t = TestGame::new(2);
    let niko = t.battlefield(P0, "Niko Aris");
    t.g.objects[niko.0 as usize]
        .counters
        .insert("loyalty".into(), 3);
    let big = t.battlefield(P1, "Ancient Brontodon");
    t.g.tap(big);
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.activate(P0, niko, 1, &[obj(big)]).unwrap();
    t.resolve_all();
    let from_niko: Vec<_> = damage_events(&t)
        .into_iter()
        .filter(|(s, _, _)| *s == niko)
        .collect();
    assert_eq!(from_niko, vec![(niko, obj(big), 4)]);
    assert_eq!(t.obj_now(big).damage, 4);
}

#[test]
fn niko_delayed_trigger_sees_noncombat_damage() {
    cr!("603.7a");
    ruling!(
        "Niko Aris",
        "The delayed triggered ability created by Niko's first loyalty ability will trigger whenever the target creature deals any damage that turn, not just combat damage."
    );
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    let niko = t.battlefield(P0, "Niko Aris");
    t.g.objects[niko.0 as usize]
        .counters
        .insert("loyalty".into(), 3);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, niko, 0, &[obj(pyro)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(pyro));
    t.activate(P0, pyro, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.zone(pyro), Zone::Hand(P0));
}

#[test]
fn simulacrum_gains_life_and_deals_new_damage_for_damage_only() {
    cr!("120.1", "120.2b", "119.3");
    ruling!(
        "Simulacrum",
        "Simulacrum is the source of the damage. If an effect needs to know a characteristic of the damage's source, it will see the damage coming from Simulacrum."
    );
    ruling!(
        "Simulacrum",
        "This is not damage prevention or redirection. It actually is life gain and newly dealt damage."
    );
    ruling!(
        "Simulacrum",
        "You can't use Simulacrum on Loss of Life, just damage."
    );
    supported("Simulacrum");
    let cast_sim = |t: &mut TestGame, target: ObjectId| -> ObjectId {
        let s = t.hand(P0, "Simulacrum");
        mana(t, P0, ManaType::B, 2);
        let spell = t.cast(P0, s).target(target).go();
        t.resolve_all();
        spell
    };
    // 2 damage from a Shock: gain 2, and the Simulacrum spell deals 2 to the creature.
    let mut t = TestGame::new(2);
    let big = t.battlefield(P0, "Ancient Brontodon");
    shock(&mut t, P1, pl(P0));
    assert_eq!(t.life(P0), 18);
    let s = cast_sim(&mut t, big);
    assert_eq!(t.life(P0), 20);
    let dealt: Vec<_> = damage_events(&t)
        .into_iter()
        .filter(|(_, to, _)| *to == obj(big))
        .collect();
    assert_eq!(dealt, vec![(s, obj(big), 2)]);
    // The damage dealt to P0 wasn't undone: a second Simulacrum sees it too.
    cast_sim(&mut t, big);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.obj_now(big).damage, 4);
    // Loss of life isn't damage.
    let mut t = TestGame::new(2);
    let big = t.battlefield(P0, "Ancient Brontodon");
    lose(&mut t, P0, 3);
    cast_sim(&mut t, big);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.obj_now(big).damage, 0);
}

#[test]
fn knollspine_dragon_counts_damage_dealt_not_prevented_or_life_lost() {
    cr!("615.1", "603.3d", "608.2d");
    ruling!(
        "Knollspine Dragon",
        "This ability counts the total amount of damage (both combat and noncombat) dealt to the targeted opponent by all sources (including ones you controlled and ones you didn't) over the course of the turn. Damage that was prevented or replaced doesn't count. Damage that resolved but didn't cause loss of life (due to Worship, for example) will count."
    );
    ruling!(
        "Knollspine Dragon",
        "You target an opponent when the ability triggers. You don't decide whether you want to discard and draw until the ability resolves."
    );
    supported("Knollspine Dragon");
    supported("Worship");
    supported("Shieldmate's Blessing");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Worship");
    let p1_creature = t.battlefield(P1, "Prodigal Pyromancer");
    t.g.players[1].life = 3;
    // A 3-point shield prevents one Shock and half of a second (1 dealt), then Lava Axe
    // deals 5 (Worship keeps P1 at 1), then P1's own creature deals 1.
    shield(&mut t, pl(P1));
    shock(&mut t, P0, pl(P1));
    assert_eq!(t.life(P1), 3);
    shock(&mut t, P0, pl(P1));
    assert_eq!(t.life(P1), 2);
    let axe = t.hand(P0, "Lava Axe");
    mana(&mut t, P0, ManaType::R, 5);
    t.cast(P0, axe).target(pl(P1)).go();
    t.resolve_all();
    damage(&mut t, p1_creature, pl(P1), 1);
    assert_eq!(t.life(P1), 1);
    // Damage dealt: 1 + 5 + 1 = 7.
    let dragon = t.hand(P0, "Knollspine Dragon");
    t.hand(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 7);
    t.cast(P0, dragon).go();
    t.resolve();
    // The trigger is on the stack targeting P1; no discard-and-draw choice yet.
    assert_eq!(t.stack_len(), 1);
    let yes_nos = |t: &TestGame| {
        t.asked()
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
            .count()
    };
    assert_eq!(yes_nos(&t), 0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_nos(&t), 1);
    assert_eq!(t.hand_size(P0), 7);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn whipkeeper_and_discordant_spirit_ignore_prevented_damage() {
    cr!("615.1", "608.2h");
    ruling!(
        "Whipkeeper",
        "This counts all damage that was dealt during the turn. This does not include any damage that was prevented."
    );
    ruling!(
        "Discordant Spirit",
        "This will not count damage that was prevented."
    );
    supported("Whipkeeper");
    supported("Discordant Spirit");
    // Whipkeeper: Brontodon was dealt 2 by a Shock, then a 3-point shield prevented a
    // second Shock and 1 of a third: 3 dealt in all.
    let mut t = TestGame::new(2);
    let whip = t.battlefield(P0, "Whipkeeper");
    let big = t.battlefield(P1, "Ancient Brontodon");
    shock(&mut t, P0, obj(big));
    shield(&mut t, obj(big));
    shock(&mut t, P0, obj(big));
    shock(&mut t, P0, obj(big));
    assert_eq!(t.obj_now(big).damage, 3);
    t.activate(P0, whip, 0, &[obj(big)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(big).damage, 6);
    // Discordant Spirit (P0's) on P1's turn: 2 dealt to P0, another 2 prevented.
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Discordant Spirit");
    t.set_step(P1, Step::PrecombatMain);
    shock(&mut t, P1, pl(P0));
    shield(&mut t, pl(P0));
    shock(&mut t, P1, pl(P0));
    assert_eq!(t.life(P0), 18);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(spirit, "+1/+1"), 2);
}

#[test]
fn notorious_throng_counts_damage_to_opponents_who_left() {
    cr!("800.4a", "608.2h");
    ruling!(
        "Notorious Throng",
        "To determine the value of X, this spell looks back over the course of the turn and counts all damage dealt by all sources to players who are currently your opponents, as well as damage dealt by all sources to players who were your opponents at the time they left the game."
    );
    supported("Notorious Throng");
    let mut t = TestGame::new(3);
    let src = t.battlefield(P0, "Prodigal Pyromancer");
    let theirs = t.battlefield(P2, "Prodigal Pyromancer");
    damage(&mut t, src, pl(P1), 2);
    damage(&mut t, theirs, pl(P2), 3);
    // Damage to P0 itself doesn't count.
    damage(&mut t, theirs, pl(P0), 4);
    crate::r_p079_common::concede(&mut t, P2);
    assert!(t.has_lost(P2));
    let nt = t.hand(P0, "Notorious Throng");
    mana(&mut t, P0, ManaType::U, 4);
    t.cast(P0, nt).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Faerie").len(), 5);
}

#[test]
fn proft_counts_draws_before_it_entered_and_as_it_resolves() {
    cr!("608.2h", "603.4");
    ruling!(
        "Proft's Eidetic Memory",
        "Proft's Eidetic Memory's last ability looks at how many cards you've drawn this turn, even if it wasn't on the battlefield when you drew those cards."
    );
    ruling!(
        "Proft's Eidetic Memory",
        "The value of X is determined only once, as Proft's Eidetic Memory's last ability resolves."
    );
    supported("Proft's Eidetic Memory");
    let mut t = TestGame::new(2);
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.battlefield(P0, "Proft's Eidetic Memory");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A third draw in response counts: X = 3 - 1.
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    // Drawing afterwards doesn't change it.
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}

#[test]
fn thalisse_counts_every_token_created_this_turn() {
    cr!("111.1", "608.2h");
    ruling!(
        "Thalisse, Reverent Medium",
        "Thalisse's ability counts all tokens you created that turn prior to the ability resolving, including both noncreature tokens and creature tokens. It doesn't matter if those tokens are still on the battlefield or still under your control."
    );
    supported("Thalisse, Reverent Medium");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thalisse, Reverent Medium");
    let rta = t.hand(P0, "Raise the Alarm");
    mana(&mut t, P0, ManaType::W, 2);
    t.cast(P0, rta).go();
    t.resolve_all();
    crate::r_p216_common::create_tokens(&mut t, P0, "Treasure", 1);
    // An opponent's token doesn't count.
    crate::r_p216_common::create_tokens(&mut t, P1, "Treasure", 1);
    let soldiers = with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 2);
    kill(&mut t, soldiers[0]);
    crate::r_p117_common::gain_control(&mut t, P1, soldiers[1]);
    assert_eq!(tokens(&t, P0).len(), 1);
    end_step(&mut t);
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 3);
}

#[test]
fn ellyn_counts_tokens_you_created_that_another_player_controls_now() {
    // The player who creates a token is its owner (CR 111.2), so "tokens you created this
    // turn" still counts a token another player has gained control of (as Thalisse's
    // ruling says), in every card with this wording.
    cr!("111.2", "608.2h");
    ruling!(
        "Ellyn Harbreeze, Busybody",
        "It doesn't matter if those tokens are still on the battlefield or under your control."
    );
    supported("Ellyn Harbreeze, Busybody");
    let mut t = TestGame::new(2);
    let ellyn = t.battlefield(P0, "Ellyn Harbreeze, Busybody");
    crate::r_p216_common::create_tokens(&mut t, P0, "Treasure", 3);
    crate::r_p216_common::create_tokens(&mut t, P1, "Treasure", 2);
    let mine = with_subtype(&t, P0, "Treasure");
    crate::r_p117_common::gain_control(&mut t, P1, mine[0]);
    kill(&mut t, mine[1]);
    let from = t.asked().len();
    t.activate(P0, ellyn, 0, &[]).unwrap();
    t.resolve_all();
    let looked: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.len()),
            _ => None,
        })
        .collect();
    assert_eq!(looked, vec![3]);
    // Vazi's {T} ability (the card's last ability doesn't compile) counts the same way.
    let mut t = TestGame::new(2);
    let vazi = t.battlefield(P0, "Vazi, Keen Negotiator");
    crate::r_p216_common::create_tokens(&mut t, P0, "Treasure", 2);
    let mine = with_subtype(&t, P0, "Treasure");
    crate::r_p117_common::gain_control(&mut t, P1, mine[0]);
    t.activate(P0, vazi, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P1, "Treasure").len(), 1 + 2);
}

#[test]
fn fraying_sanity_counts_as_each_trigger_resolves() {
    cr!("608.2h", "701.17a");
    ruling!(
        "Fraying Sanity",
        "if three Fraying Sanity Auras are attached to one player who had four cards put into their graveyard this turn, X will be four for the first ability to resolve, eight for the second, and sixteen for the third."
    );
    supported("Fraying Sanity");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        let a = t.battlefield(P0, "Fraying Sanity");
        assert!(t.g.attach(a, pl(P1)));
    }
    t.g.mill(P1, 4);
    t.g.flush_events();
    assert_eq!(t.library_size(P1), 26);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 3);
    for left in [22, 14, 0] {
        t.resolve();
        assert_eq!(t.library_size(P1), left);
    }
    assert_eq!(t.graveyard_size(P1), 30);
}

#[test]
fn lilianas_standard_bearer_counts_itself_if_it_dies_in_response() {
    cr!("608.2h", "700.4");
    ruling!(
        "Liliana's Standard Bearer",
        "The value of X is determined only as the triggered ability resolves. If Liliana's Standard Bearer dies while that ability's on the stack, it will count itself."
    );
    supported("Liliana's Standard Bearer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // An opponent's creature dying doesn't count.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, bears);
    kill(&mut t, theirs);
    let lsb = t.hand(P0, "Liliana's Standard Bearer");
    mana(&mut t, P0, ManaType::B, 3);
    t.cast(P0, lsb).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, lsb);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}
