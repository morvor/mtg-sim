//! Rulings batch P126 — attack-counting and "grows with others" triggers (CR 603.2,
//! 608.2h), quest counters (Luminarch Ascension), rack effects with intervening "if"
//! clauses (CR 603.4), raid (CR 702 "if you attacked this turn"), rainbow and devotion
//! lands as mana abilities (CR 605, 106), rampage-like bonuses, excess damage with
//! deathtouch (CR 120.4a, 702.2c), and Skull Rend.

use crate::r_p076_common::mana;
use crate::r_p108_common::two_headed_giant;
use crate::r_s01_common::{attack_with, supported, tokens};
use crate::r_s03_common::to_blockers;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::has_kw;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

// ---------------------------------------------------------------------------
// Quadratic
// ---------------------------------------------------------------------------

#[test]
fn shared_animosity_counts_creatures_not_creature_types() {
    cr!("603.2", "205.3", "702.73a");
    ruling!(
        "Shared Animosity",
        "This ability counts creatures, not creature types. For example, if you attack with five creatures"
    );
    supported("Shared Animosity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shared Animosity");
    let names = [
        ("Elvish Visionary", 3),
        ("Elvish Warrior", 2),
        ("Akki Rockspeaker", 2),
        ("Fire Elemental", 1),
        ("Changeling Outcast", 4),
    ];
    let ids: Vec<(ObjectId, i32, i32)> = names
        .iter()
        .map(|(n, bonus)| {
            let id = t.battlefield(P0, n);
            (id, t.pt(id).0, *bonus)
        })
        .collect();
    let decl: Vec<_> = ids.iter().map(|(id, _, _)| (*id, Entity::Player(P1))).collect();
    attack_with(&mut t, &decl);
    assert_eq!(t.stack_len(), 5, "it triggers once per attacking creature");
    t.resolve_all();
    for (i, (id, base, bonus)) in ids.iter().enumerate() {
        assert_eq!(t.pt(*id).0, base + bonus, "{}", names[i].0);
    }
}

#[test]
fn shared_animosity_in_two_headed_giant() {
    cr!("810.7", "805.10a", "603.2");
    ruling!(
        "Shared Animosity",
        "In a Two-Headed Giant game, only creatures you control trigger the ability and get the bonus, but your teammate's attacking creatures are included in the calculation of those bonuses."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Shared Animosity");
    let mine = t.battlefield(P0, "Elvish Warrior");
    let theirs = t.battlefield(P1, "Elvish Visionary");
    t.set_step(P0, Step::BeginningOfCombat);
    // The team's attack is declared together (by its primary player).
    attack_with(
        &mut t,
        &[(mine, Entity::Player(P2)), (theirs, Entity::Player(P2))],
    );
    assert!(t.g.combat.as_ref().unwrap().attackers.len() == 2, "both attack");
    assert_eq!(t.stack_len(), 1, "only P0's creature triggers it");
    t.resolve_all();
    assert_eq!(t.pt(mine).0, 3);
    assert_eq!(t.pt(theirs).0, 1);
}

#[test]
fn minn_uses_the_illusions_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Minn, Wily Illusionist",
        "Minn's last ability uses the Illusion's power the last time it existed on the battlefield"
    );
    supported("Minn, Wily Illusionist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Minn, Wily Illusionist");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.resolve_all();
    let illusion = tokens(&t, P0)[0];
    // Giant Growth: it's a 4/4 when it dies.
    let growth = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, growth).target(illusion).go();
    t.resolve_all();
    assert_eq!(t.pt(illusion).0, 4);
    let giant = t.hand(P0, "Hill Giant");
    let angel = t.hand(P0, "Serra Angel");
    t.g.destroy(illusion, None);
    t.g.flush_events();
    t.settle();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    assert!(t.on_battlefield(giant), "mana value 4 <= its last power 4");
    let offered_angel = t.asked()[from..].iter().any(|(_, d)| match d {
        Decision::ChooseEntities { candidates, .. } => candidates.contains(&obj(angel)),
        _ => false,
    });
    assert!(!offered_angel, "Serra Angel (mana value 5) isn't allowed");
}

#[test]
fn prismabasher_counts_x_twice() {
    cr!("601.2c", "603.3d", "608.2h");
    ruling!(
        "Prismabasher",
        "The value of X is calculated when Prismabasher's last ability triggers for the purpose of defining the maximum number of target creatures. It is calculated again when the ability resolves"
    );
    supported("Prismabasher");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let lions = t.battlefield(P0, "Savannah Lions");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Four colors as it triggers (green, red, white, black): up to four targets.
    t.answer_targets(P0, &[obj(giant), obj(lions), obj(hawk), obj(bears)]);
    let basher = enter(&mut t, P0, "Prismabasher");
    assert_eq!(t.stack_len(), 1);
    // The black permanent leaves: three colors as it resolves.
    move_to(&mut t, hawk, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.pt(giant), (6, 6));
    assert_eq!(t.pt(lions), (5, 4));
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(basher), (6, 6));
}

#[test]
fn alistair_pays_as_the_ability_resolves() {
    cr!("608.2c", "118.12");
    ruling!(
        "Alistair, the Brigadier",
        "You choose whether to pay {8} as Alistair's second ability resolves."
    );
    supported("Alistair, the Brigadier");
    let mut t = TestGame::new(2);
    let alistair = t.battlefield(P0, "Alistair, the Brigadier");
    t.battlefield(P0, "Ornithopter");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(alistair, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    // Mana arrives after the trigger was put on the stack; it's paid as it resolves.
    mana(&mut t, P0, ManaType::C, 8);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
    // Two historic permanents: Alistair (legendary) and Ornithopter (artifact).
    assert_eq!(t.pt(alistair), (5, 5));
}

// ---------------------------------------------------------------------------
// Quest
// ---------------------------------------------------------------------------

#[test]
fn luminarch_ascension_checks_life_lost_this_turn() {
    cr!("603.4", "119.3");
    ruling!(
        "Luminarch Ascension",
        "Luminarch Ascension's first ability cares only whether you lost life this turn, even if Luminarch Ascension wasn't on the battlefield when that happened. It doesn't care how much you lost, whether you also gained life, or even whether you gained more life than you lost."
    );
    supported("Luminarch Ascension");
    for lose in [false, true] {
        let mut t = TestGame::new(2);
        t.advance_to(P1, Step::Upkeep);
        if lose {
            t.g.lose_life(P0, 1);
            t.g.gain_life(P0, 5);
            t.g.flush_events();
        }
        let la = t.battlefield(P0, "Luminarch Ascension");
        t.answer_yes(P0, true);
        t.advance_to(P1, Step::End);
        t.resolve_all();
        assert_eq!(t.counters(la, "quest"), if lose { 0 } else { 1 });
    }
}

#[test]
fn luminarch_ascension_can_activate_in_the_same_end_step() {
    cr!("602.5b", "122.1");
    ruling!(
        "Luminarch Ascension",
        "You can activate Luminarch Ascension's last ability during the end step in which it receives its fourth quest counter."
    );
    let mut t = TestGame::new(2);
    let la = t.battlefield(P0, "Luminarch Ascension");
    t.g.add_counters(obj(la), "quest", 3, None);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(la, "quest"), 4);
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, la, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

// ---------------------------------------------------------------------------
// Rack effects
// ---------------------------------------------------------------------------

fn hand_of(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.hand(p, n)).collect()
}

#[test]
fn rack_triggers_check_the_hand_twice() {
    cr!("603.4");
    ruling!(
        "Shrieking Affliction",
        "Shrieking Affliction’s ability will trigger only if an opponent begins their upkeep with one or fewer cards in hand. The ability will check the number of cards in that player’s hand again when it tries to resolve. If that player has two or more cards in hand at that time, that player won’t lose life."
    );
    ruling!(
        "Lavaborn Muse",
        "The number of cards in hand is checked both at the beginning of upkeep and when the triggered ability resolves."
    );
    // (card, the most cards for the trigger, life lost)
    for (name, max, loss) in [("Shrieking Affliction", 1, 3), ("Lavaborn Muse", 2, 3)] {
        supported(name);
        // Too many cards: no trigger.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        hand_of(&mut t, P1, &vec!["Island"; max + 1]);
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}");
        // It triggers; the player has more cards as it resolves: nothing happens.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        hand_of(&mut t, P1, &vec!["Island"; max]);
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{name}");
        t.hand(P1, "Island");
        t.resolve_all();
        assert_eq!(t.life(P1), 20, "{name}");
        // It triggers and still applies.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        hand_of(&mut t, P1, &vec!["Island"; max]);
        t.advance_to(P1, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - loss, "{name}");
    }
}

#[test]
fn bandits_talent_either_one_nonland_or_two_cards() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Bandit's Talent",
        "Opponents can discard either one nonland card or two cards which may or may not be nonland cards. If they really want to, they can discard two nonland cards."
    );
    supported("Bandit's Talent");
    // Discard one nonland card.
    let mut t = TestGame::new(2);
    hand_of(&mut t, P1, &["Grizzly Bears", "Hill Giant", "Island"]);
    t.answer_yes(P1, true);
    enter(&mut t, P0, "Bandit's Talent");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
    assert!(t.g.player(P1).graveyard.iter().all(|c| !t.g.obj(*c).is(CardType::Land)));
    // Discard two cards, both nonland.
    let mut t = TestGame::new(2);
    let h = hand_of(&mut t, P1, &["Grizzly Bears", "Hill Giant", "Island"]);
    t.answer_yes(P1, false);
    t.answer_choose(P1, &[obj(h[0]), obj(h[1])]);
    enter(&mut t, P0, "Bandit's Talent");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.in_hand(P1, "Island"));
}

// ---------------------------------------------------------------------------
// Raid
// ---------------------------------------------------------------------------

#[test]
fn wingmate_roc_counts_attackers_as_it_resolves() {
    cr!("608.2h", "506.4");
    ruling!(
        "Wingmate Roc",
        "Count the number of attacking creatures when the last ability resolves, including Wingmate Roc itself if it's still on the battlefield"
    );
    supported("Wingmate Roc");
    let mut t = TestGame::new(2);
    let roc = t.battlefield(P0, "Wingmate Roc");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(
        &mut t,
        &[
            (roc, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
    );
    move_to(&mut t, b, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Wingmate Roc itself left: only the others count.
    let mut t = TestGame::new(2);
    let roc = t.battlefield(P0, "Wingmate Roc");
    let a = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(roc, Entity::Player(P1)), (a, Entity::Player(P1))]);
    move_to(&mut t, roc, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn nightsquad_commando_checks_only_whether_you_attacked() {
    cr!("508.1", "603.4");
    ruling!(
        "Nightsquad Commando",
        "Nightsquad Commando’s ability checks only whether you declared an attacking creature during the turn. It doesn’t trigger once for each such creature, and it doesn’t check whether any attacking creatures dealt damage."
    );
    supported("Nightsquad Commando");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Darksteel Myr");
    // Both attack; the attacker that's blocked is removed and the other dealt no damage.
    to_blockers(
        &mut t,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
        &[(blocker, a)],
    );
    move_to(&mut t, b, Zone::Graveyard(P0));
    move_to(&mut t, a, Zone::Graveyard(P0));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 20);
    enter(&mut t, P0, "Nightsquad Commando");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

/// P0 has Perforating Artist and attacked this turn (with an Ornithopter, dealing no
/// damage); it's P0's end step and the raid trigger resolves.
fn artist_end_step(t: &mut TestGame, defender: PlayerId) {
    t.battlefield(P0, "Perforating Artist");
    let orn = t.battlefield(P0, "Ornithopter");
    t.attack(&[(orn, Entity::Player(defender))], &[]);
    t.advance_to(P0, Step::End);
}

#[test]
fn perforating_artist_opponent_may_always_lose_life() {
    cr!("608.2c", "118.12", "701.9a", "701.21a");
    ruling!(
        "Perforating Artist",
        "Your opponent can always choose to lose 3 life, even if they have cards to discard or nonland permanents to sacrifice."
    );
    supported("Perforating Artist");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.answer_yes(P1, false);
    artist_end_step(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").len() == 1);
}

#[test]
fn perforating_artist_in_two_headed_giant() {
    cr!("810.9", "608.2c");
    ruling!(
        "Perforating Artist",
        "In a Two-Headed Giant game, Perforating Artist's last ability causes both opponents to sacrifice a nonland permanent, discard a card, or lose 3 life."
    );
    let mut t = two_headed_giant();
    let life = t.life(P2);
    t.hand(P2, "Hill Giant");
    t.answer_yes(P2, true);
    t.answer_yes(P3, false);
    artist_end_step(&mut t, P2);
    t.resolve_all();
    // P2 discarded; P3 (nothing to sacrifice or discard) lost 3 life for the team.
    assert!(t.in_graveyard(P2, "Hill Giant"));
    assert_eq!(t.life(P2), life - 3);
}

// ---------------------------------------------------------------------------
// Rainbow lands
// ---------------------------------------------------------------------------

#[test]
fn exotic_orchard_ignores_riders_on_opponents_mana() {
    cr!("106.7", "106.6");
    ruling!(
        "Exotic Orchard",
        "Exotic Orchard doesn't care about any restrictions or riders your opponents' lands (such as Ancient Ziggurat or Hall of the Bandit Lord) put on the mana they produce. It just cares about colors of mana."
    );
    supported("Exotic Orchard");
    let mut t = TestGame::new(2);
    let orchard = t.battlefield(P0, "Exotic Orchard");
    t.battlefield(P1, "Ancient Ziggurat");
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, orchard, 0, &[]).unwrap();
    let m = &t.g.player(P0).mana_pool.mana[0];
    assert_eq!(m.ty, ManaType::G);
    assert_eq!(m.restriction, None);
    // It pays for a noncreature spell.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn rainbow_mana_abilities_dont_use_the_stack() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Tarnation Vista",
        "Tarnation Vista’s last ability is a mana ability. It doesn’t use the stack and can’t be responded to."
    );
    ruling!(
        "Nykthos, Shrine to Nyx",
        "The second ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Tarnation Vista");
    supported("Nykthos, Shrine to Nyx");
    // Tarnation Vista with a red and a green monocolored permanent.
    let mut t = TestGame::new(2);
    let vista = t.battlefield(P0, "Tarnation Vista");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 1);
    assert_eq!(t.activate(P0, vista, 1, &[]).unwrap(), None);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 1);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 1);
    // Nykthos with devotion to red 2 (Shivan Dragon).
    let mut t = TestGame::new(2);
    let nykthos = t.battlefield(P0, "Nykthos, Shrine to Nyx");
    t.battlefield(P0, "Shivan Dragon");
    mana(&mut t, P0, ManaType::C, 2);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    assert_eq!(t.activate(P0, nykthos, 1, &[]).unwrap(), None);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 2);
}

#[test]
fn tarnation_vista_makes_at_most_five_mana() {
    cr!("105.1", "105.2c");
    ruling!(
        "Tarnation Vista",
        "The five colors are white, blue, black, red, and green, and as such, Tarnation Vista’s last ability can produce five mana at most."
    );
    let mut t = TestGame::new(2);
    let vista = t.battlefield(P0, "Tarnation Vista");
    for n in [
        "Savannah Lions",
        "Savannah Lions",
        "Air Elemental",
        "Vampire Nighthawk",
        "Hill Giant",
        "Grizzly Bears",
        // Multicolored: not counted.
        "Alistair, the Brigadier",
    ] {
        t.battlefield(P0, n);
    }
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, vista, 1, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 5);
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        assert_eq!(t.g.player(P0).mana_pool.count(ty), 1);
    }
}

#[test]
fn public_thoroughfare_can_tap_itself_for_its_trigger() {
    cr!("603.3", "608.2c");
    ruling!(
        "Public Thoroughfare",
        "In the unusual case where Public Thoroughfare is untapped as its triggered ability is resolving, you can tap it for its own triggered ability."
    );
    supported("Public Thoroughfare");
    let mut t = TestGame::new(2);
    let pt = enter(&mut t, P0, "Public Thoroughfare");
    assert!(t.obj_now(pt).tapped);
    assert_eq!(t.stack_len(), 1);
    // Untapped before the trigger resolves; it's the only artifact or land.
    t.g.untap(t.g.current(pt));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(t.g.current(pt))]);
    t.resolve_all();
    assert!(t.on_battlefield(pt));
    assert!(t.obj_now(pt).tapped);
}

#[test]
fn thran_quarry_triggers_at_every_end_step() {
    cr!("513.2", "603.4");
    ruling!("Thran Quarry", "Triggers at the end of every player’s turn.");
    supported("Thran Quarry");
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Thran Quarry");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(q));
}

#[test]
fn meteor_crater_cant_choose_colorless() {
    cr!("105.1", "106.5");
    ruling!(
        "Meteor Crater",
        "You can’t choose “colorless”. You have to choose one of the five colors."
    );
    supported("Meteor Crater");
    let mut t = TestGame::new(2);
    let crater = t.battlefield(P0, "Meteor Crater");
    t.battlefield(P0, "Ornithopter");
    t.activate(P0, crater, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 0);
}

// ---------------------------------------------------------------------------
// Excess damage, rampage-like, random discard
// ---------------------------------------------------------------------------

#[test]
fn hell_to_pay_with_deathtouch_has_more_excess_damage() {
    cr!("120.4a", "702.2c");
    ruling!(
        "Hell to Pay",
        "In some unusual cases, Hell to Pay may have deathtouch. Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage"
    );
    supported("Hell to Pay");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pestilent Spirit");
    let target = t.battlefield(P1, "Hill Giant");
    let h = t.hand(P0, "Hell to Pay");
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, h).x(3).target(target).go();
    t.resolve_all();
    assert!(!t.on_battlefield(target));
    assert_eq!(tokens(&t, P0).len(), 2, "3 damage, 1 of it lethal: 2 excess");
}

#[test]
fn rampage_like_bonuses() {
    cr!("509.1h", "611.2a", "514.2");
    ruling!(
        "Barreling Attack",
        "Both the +1/+1 and trample last only until end of turn."
    );
    ruling!("Gang of Elk", "Gang of Elk gets the bonus once for each blocker.");
    supported("Barreling Attack");
    supported("Gang of Elk");
    // Gang of Elk blocked by two creatures: +4/+4.
    let mut t = TestGame::new(2);
    let elk = t.battlefield(P0, "Gang of Elk");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(elk, Entity::Player(P1))], &[(b1, elk), (b2, elk)]);
    t.resolve_all();
    assert_eq!(t.pt(elk), (9, 8));
    // Barreling Attack: trample and +1/+1 per blocker, until end of turn.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    let ba = t.hand(P0, "Barreling Attack");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, ba).target(giant).go();
    t.resolve_all();
    assert!(has_kw(&t, giant, KeywordKind::Trample));
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(b1, giant), (b2, giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 5));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(giant), (3, 3));
    assert!(!has_kw(&t, giant, KeywordKind::Trample));
}

#[test]
fn skull_rend_needs_no_targets_and_discards_even_if_prevented() {
    cr!("115.1", "615.1", "701.9b");
    ruling!("Skull Rend", "Skull Rend doesn’t require any targets.");
    ruling!(
        "Skull Rend",
        "Each of your opponents will discard cards, even if some or all of the damage is prevented or redirected."
    );
    supported("Skull Rend");
    let mut t = TestGame::new(2);
    // All damage that would be dealt to P1 by sources P1 doesn't control is prevented.
    t.battlefield(P1, "Energy Field");
    hand_of(&mut t, P1, &["Island", "Island", "Island"]);
    let sr = t.hand(P0, "Skull Rend");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, sr).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 1);
}

// ---------------------------------------------------------------------------
// Pseudo-proliferate
// ---------------------------------------------------------------------------

#[test]
fn giving_counters_is_putting_counters() {
    cr!("122.6", "614.1a");
    ruling!(
        "Maulfist Revolutionary",
        "Effects that interact with a player getting counters or counters being placed on permanents interact with Maulfist Revolutionary’s triggered ability."
    );
    ruling!(
        "Skyship Plunderer",
        "Effects that interact with a player getting counters or counters being placed on permanents interact with Skyship Plunderer's triggered ability."
    );
    supported("Maulfist Revolutionary");
    supported("Skyship Plunderer");
    // Maulfist Revolutionary enters: Hardened Scales adds one more +1/+1 counter.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), "+1/+1", 1, None);
    t.battlefield(P0, "Hardened Scales");
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Maulfist Revolutionary");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    // Skyship Plunderer deals combat damage: the same.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), "+1/+1", 1, None);
    t.battlefield(P0, "Hardened Scales");
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    let sp = t.battlefield(P0, "Skyship Plunderer");
    t.answer_targets(P0, &[obj(bears)]);
    t.attack(&[(sp, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.counters(bears, "+1/+1"), 3);
}
