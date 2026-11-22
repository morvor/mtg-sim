//! Rulings batch P127 — abilities that return a card from its owner's graveyard and
//! trigger only while it's there: when the card must already be in the graveyard,
//! intervening "if" clauses, leaving the graveyard before resolution, entering attacking
//! without being declared, simultaneous deaths, and per-event triggers.

use crate::r_p127_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn nether_traitor_sees_a_dying_token() {
    cr!("111.7", "704.5d", "603.10a");
    ruling!(
        "Nether Traitor",
        "A token you own that dies is put into your graveyard before it ceases to exist."
    );
    supported("Nether Traitor");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Nether Traitor");
    t.lands(P0, "Swamp", 1);
    let tok = create_token(&mut t, P0, "Soldier");
    destroy(&mut t, tok);
    assert_eq!(triggers_on_stack(&t, "return ~"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(named_by(&t, P0, "Nether Traitor").len(), 1);
}

#[test]
fn nether_traitor_dying_alongside_another_creature_doesnt_trigger() {
    cr!("603.10a", "603.2");
    ruling!(
        "Nether Traitor",
        "If Nether Traitor and another creature are put into your graveyard at the same time, Nether Traitor's ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let traitor = t.battlefield(P0, "Nether Traitor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy_all(vec![traitor, bears], None, false);
    t.g.flush_events();
    t.settle();
    assert!(t.in_graveyard(P0, "Nether Traitor"));
    assert_eq!(triggers_on_stack(&t, "return ~"), 0);
}

#[test]
fn nether_traitor_triggers_for_each_creature_but_returns_once() {
    cr!("603.2c", "400.7", "608.2b");
    ruling!(
        "Nether Traitor",
        "If multiple creatures are put into your graveyard at the same time, Nether Traitor's ability triggers for each of them."
    );
    let mut t = TestGame::new(2);
    let traitor = t.graveyard(P0, "Nether Traitor");
    t.lands(P0, "Swamp", 2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.g.destroy_all(vec![a, b], None, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "return ~"), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(named_by(&t, P0, "Nether Traitor").len(), 1);
    assert!(t.on_battlefield(traitor));
}

#[test]
fn graveyard_only_activated_abilities() {
    cr!("113.6k", "602.2");
    ruling!(
        "Advanced Stitchwing",
        "Advanced Stitchwing's last ability can only be activated while it's in your graveyard."
    );
    ruling!(
        "Geralf's Masterpiece",
        "Geralf's Masterpiece's last ability can only be activated while it's in your graveyard."
    );
    ruling!(
        "Ghoulsteed",
        "Ghoulsteed's ability can only be activated while it's in your graveyard."
    );
    ruling!(
        "Haunted Dead",
        "Haunted Dead’s last ability can only be activated while it’s in your graveyard."
    );
    for name in [
        "Advanced Stitchwing",
        "Geralf's Masterpiece",
        "Ghoulsteed",
        "Haunted Dead",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 4);
        t.lands(P0, "Swamp", 4);
        for _ in 0..3 {
            t.hand(P0, "Forest");
        }
        let on_bf = t.battlefield(P0, name);
        assert!(!can_activate(&mut t, P0, on_bf), "{name} on the battlefield");
        let in_gy = t.graveyard(P0, name);
        assert!(can_activate(&mut t, P0, in_gy), "{name} in the graveyard");
        activate_containing(&mut t, P0, in_gy, "Return").unwrap();
        t.resolve_all();
        assert!(t.on_battlefield(in_gy), "{name} returned");
        assert!(t.obj_now(in_gy).tapped, "{name} returns tapped");
    }
}

#[test]
fn akuta_must_be_in_the_graveyard_as_upkeep_begins() {
    cr!("503.1a", "603.4");
    ruling!(
        "Akuta, Born of Ash",
        "Akuta must already be in your graveyard as your upkeep begins in order for its ability to trigger."
    );
    supported("Akuta, Born of Ash");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Swamp");
    for _ in 0..3 {
        t.hand(P0, "Forest");
    }
    t.g.players[1].hand.clear();
    let akuta = t.hand(P0, "Akuta, Born of Ash");
    t.advance_to(P0, Step::Upkeep);
    let akuta = move_to(&mut t, akuta, Zone::Graveyard(P0)).unwrap();
    assert_eq!(triggers_on_stack(&t, "sacrifice a Swamp"), 0);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.g.players[1].hand.clear();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(akuta));
}

#[test]
fn akuta_checks_each_opponent_in_multiplayer() {
    cr!("603.4", "102.1");
    ruling!("Akuta, Born of Ash", "In multiplayer games, it checks for each opponent.");
    for (p2_hand, triggers) in [(5usize, false), (1, true)] {
        let mut t = TestGame::new(3);
        t.graveyard(P0, "Akuta, Born of Ash");
        t.battlefield(P0, "Swamp");
        t.set_step(P2, Step::End);
        t.g.players[0].hand.clear();
        t.g.players[1].hand.clear();
        t.g.players[2].hand.clear();
        for _ in 0..3 {
            t.hand(P0, "Forest");
        }
        t.hand(P1, "Forest");
        for _ in 0..p2_hand {
            t.hand(P2, "Forest");
        }
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::Upkeep);
        // P0 drew no card yet: 3 cards against 1 and 5 (or 1).
        t.resolve_all();
        assert_eq!(
            named_by(&t, P0, "Akuta, Born of Ash").len(),
            triggers as usize,
            "P2 hand {p2_hand}"
        );
    }
}

/// A 3-player game in P0's declare attackers step with three attackers at P1, the
/// returning card in P0's graveyard, and Fervent Charge ("Whenever a creature you control
/// attacks") on the battlefield.
fn attack_with_three(card: &str, mana: &[(&str, usize)]) -> (TestGame, ObjectId) {
    supported(card);
    let mut t = TestGame::new(3);
    let c = t.graveyard(P0, card);
    t.battlefield(P0, "Fervent Charge");
    for (land, n) in mana {
        t.lands(P0, land, *n);
    }
    let atk: Vec<ObjectId> = ["Grizzly Bears", "Hill Giant", "Raging Goblin"]
        .iter()
        .map(|n| t.battlefield(P0, n))
        .collect();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let at: Vec<(ObjectId, Entity)> = atk.iter().map(|a| (*a, Entity::Player(P1))).collect();
    attack_with(&mut t, &at);
    // Fervent Charge's three triggers and the return trigger.
    assert_eq!(triggers_on_stack(&t, "gets +2/+2"), 3);
    t.resolve_all();
    (t, c)
}

#[test]
fn phoenixes_enter_attacking_a_chosen_opponent_without_attacking() {
    cr!("508.4", "508.3a", "506.4");
    ruling!(
        "Warcry Phoenix",
        "As Warcry Phoenix returns to the battlefield because of its triggered ability, you choose which opponent or opposing planeswalker it's attacking."
    );
    ruling!(
        "Warcry Phoenix",
        "If Warcry Phoenix enters the battlefield attacking, it wasn't declared as an attacking creature that turn."
    );
    ruling!(
        "Phoenix Chick",
        "As Phoenix Chick returns to the battlefield because of its triggered ability, you choose which opponent or opposing planeswalker it's attacking."
    );
    ruling!(
        "Phoenix Chick",
        "If Phoenix Chick enters the battlefield attacking, it wasn't declared as an attacking creature that turn."
    );
    for (card, mana) in [
        ("Warcry Phoenix", vec![("Mountain", 3)]),
        ("Phoenix Chick", vec![("Mountain", 2)]),
    ] {
        let (t, c) = attack_with_three(card, &mana);
        assert!(t.on_battlefield(c), "{card} returned");
        assert!(t.g.is_attacking(t.g.current(c)));
        assert_eq!(
            t.g.combat.as_ref().unwrap().attack_target(t.g.current(c)),
            Some(Entity::Player(P2)),
            "{card}"
        );
        // Fervent Charge didn't trigger for it (Warcry Phoenix is a 2/2, Phoenix Chick a
        // 1/1 with a +1/+1 counter).
        assert_eq!(t.pt(c).0, 2, "{card}");
    }
}

#[test]
fn legendary_and_rat_returners_enter_attacking_a_chosen_player() {
    cr!("508.4", "508.3a");
    ruling!(
        "Interceptor, Shadow's Hound",
        "As Interceptor returns to the battlefield because of its triggered ability, you choose which player, planeswalker, or battle it's attacking."
    );
    ruling!(
        "Interceptor, Shadow's Hound",
        "Although Interceptor's last ability causes it to enter attacking, it was never declared as an attacking creature."
    );
    ruling!(
        "Persistent Marshstalker",
        "As Persistent Marshstalker returns to the battlefield because of its triggered ability, you choose which player, planeswalker, or battle it's attacking."
    );
    ruling!(
        "Persistent Marshstalker",
        "Although Persistent Marshstalker's last ability causes it to enter attacking, it was never declared as an attacking creature."
    );
    for (card, attacker, lands) in [
        ("Interceptor, Shadow's Hound", "Ebondeath, Dracolich", 3usize),
        ("Persistent Marshstalker", "Persistent Marshstalker", 3),
    ] {
        supported(card);
        let mut t = TestGame::new(3);
        let c = t.graveyard(P0, card);
        if card == "Persistent Marshstalker" {
            for _ in 0..7 {
                t.graveyard(P0, "Forest");
            }
        }
        t.battlefield(P0, "Fervent Charge");
        t.lands(P0, "Swamp", lands);
        let a = t.battlefield(P0, attacker);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Player(P2)]);
        attack_with(&mut t, &[(a, Entity::Player(P1))]);
        t.resolve_all();
        let now = t.g.current(c);
        assert!(t.on_battlefield(c), "{card} returned");
        assert_eq!(
            t.g.combat.as_ref().unwrap().attack_target(now),
            Some(Entity::Player(P2)),
            "{card}"
        );
        let printed = mtg_engine::card::card(card).front().chars.power.unwrap_or(0);
        assert!(t.pt(c).0 < printed + 2, "{card} got Fervent Charge's bonus");
    }
}

#[test]
fn arclight_phoenix_can_attack_the_combat_it_returns() {
    cr!("507.1", "508.1a");
    ruling!(
        "Arclight Phoenix",
        "Because the beginning of combat step is before attackers are declared, you can attack with Arclight Phoenix during the same combat it returns to the battlefield."
    );
    supported("Arclight Phoenix");
    let mut t = TestGame::new(2);
    let ph = t.graveyard(P0, "Arclight Phoenix");
    for _ in 0..3 {
        cast_new(&mut t, P0, "Opt", &[]);
        t.resolve_all();
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(ph));
    let life = t.life(P1);
    t.attack(&[(t.g.current(ph), Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), life - 3);
}

#[test]
fn sekki_prevents_all_damage_and_makes_a_token_per_point() {
    cr!("615.1a", "614.1a");
    ruling!(
        "Sekki, Seasons' Guide",
        "Creature tokens are put onto the battlefield even if there are no more +1/+1 counters to remove."
    );
    supported("Sekki, Seasons' Guide");
    let mut t = TestGame::new(2);
    let sekki = t.enter(P0, "Sekki, Seasons' Guide");
    assert_eq!(t.counters(sekki, counters::PLUS1), 8);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 9, sekki);
    t.resolve_all();
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Spirit"), 9);
    assert!(t.in_graveyard(P0, "Sekki, Seasons' Guide"));
}

#[test]
fn sekki_with_extra_toughness_still_makes_tokens() {
    cr!("615.1a", "704.5f");
    ruling!(
        "Sekki, Seasons' Guide",
        "Increasing Sekki's toughness with a card like Indomitable Will allows Sekki to prevent damage and make tokens even it if has no +1/+1 counters to be removed."
    );
    let mut t = TestGame::new(2);
    let sekki = t.battlefield(P0, "Sekki, Seasons' Guide");
    attach_new(&mut t, P0, "Indomitable Will", sekki);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 3, sekki);
    t.resolve_all();
    assert!(t.on_battlefield(sekki));
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Spirit"), 3);
    assert_eq!(t.obj_now(sekki).damage, 0);
}

#[test]
fn cult_conscript_counts_a_dying_token() {
    cr!("111.7", "602.5b");
    ruling!(
        "Cult Conscript",
        "If a non-Skeleton creature token died this turn, it satisfies the activation restriction."
    );
    supported("Cult Conscript");
    let mut t = TestGame::new(2);
    let cc = t.graveyard(P0, "Cult Conscript");
    t.lands(P0, "Swamp", 2);
    assert!(!can_activate(&mut t, P0, cc));
    let tok = create_token(&mut t, P0, "Soldier");
    destroy(&mut t, tok);
    assert!(can_activate(&mut t, P0, cc));
}

#[test]
fn deathmist_raptor_must_be_in_the_graveyard_when_turned_face_up() {
    cr!("603.2", "708.8");
    ruling!(
        "Deathmist Raptor",
        "Deathmist Raptor's second ability triggers only if Deathmist Raptor is in the graveyard when a permanent you control is turned face up."
    );
    supported("Deathmist Raptor");
    for in_gy in [false, true] {
        let mut t = TestGame::new(2);
        let raptor = if in_gy {
            t.graveyard(P0, "Deathmist Raptor")
        } else {
            t.hand(P0, "Deathmist Raptor")
        };
        let fd = crate::r_s12_common::morph(&mut t, P0, "Ashcloud Phoenix");
        t.lands(P0, "Mountain", 6);
        assert!(crate::r_s11_common::turn_face_up(&mut t, P0, fd));
        t.settle();
        assert_eq!(
            triggers_on_stack(&t, "return ~"),
            in_gy as usize,
            "in graveyard: {in_gy}"
        );
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.on_battlefield(raptor), in_gy);
    }
}

#[test]
fn leaving_the_graveyard_before_resolution_stops_the_return() {
    cr!("400.7", "608.2b");
    ruling!(
        "Ashcloud Phoenix",
        "If Ashcloud Phoenix leaves the graveyard before its \"dies\" ability resolves, it won't return to the battlefield."
    );
    ruling!(
        "Boneyard Scourge",
        "If Boneyard Scourge leaves your graveyard before its triggered ability resolves, it won't be returned to the battlefield."
    );
    ruling!(
        "Narcomoeba",
        "If Narcomoeba is removed from your graveyard before its ability resolves, you can't put it onto the battlefield."
    );
    // Ashcloud Phoenix dies.
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Ashcloud Phoenix");
    destroy(&mut t, ph);
    assert_eq!(t.stack_len(), 1);
    exile_now(&mut t, ph);
    t.resolve_all();
    assert!(is_in(&t, ph, Zone::Exile));
    // Boneyard Scourge: a Dragon P0 controls dies.
    supported("Boneyard Scourge");
    let mut t = TestGame::new(2);
    let bs = t.graveyard(P0, "Boneyard Scourge");
    t.lands(P0, "Swamp", 2);
    let dragon = t.battlefield(P0, "Shivan Dragon");
    destroy(&mut t, dragon);
    assert_eq!(t.stack_len(), 1);
    exile_now(&mut t, bs);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(is_in(&t, bs, Zone::Exile));
    // Narcomoeba is milled.
    let mut t = TestGame::new(2);
    let n = t.library_top(P0, "Narcomoeba");
    t.g.mill(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    exile_now(&mut t, n);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(is_in(&t, n, Zone::Exile));
}

#[test]
fn narcomoeba_revealed_while_milled_still_triggers() {
    cr!("603.6c", "701.17a");
    ruling!(
        "Narcomoeba",
        "If Narcomoeba is revealed or looked at while moving from your library to your graveyard (such as with Mind Funeral), its ability will trigger when it's put into your graveyard."
    );
    let mut t = TestGame::new(2);
    let n = t.library_top(P0, "Narcomoeba");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Mind Funeral", &[Entity::Player(P0)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(n));
}

#[test]
fn chronosavant_activated_repeatedly_returns_once_and_skips_each_time() {
    cr!("614.10", "608.2b");
    ruling!(
        "Chronosavant",
        "The first one that resolves will return Chronosavant to the battlefield and make you skip your next turn. The others will just make you skip turns."
    );
    supported("Chronosavant");
    let mut t = TestGame::new(2);
    let cs = t.graveyard(P0, "Chronosavant");
    t.lands(P0, "Plains", 4);
    activate_containing(&mut t, P0, cs, "Return").unwrap();
    activate_containing(&mut t, P0, cs, "Return").unwrap();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.on_battlefield(cs));
    let skips = t
        .g
        .player(P0)
        .skips
        .iter()
        .filter(|k| **k == mtg_engine::ability::StepKind::Turn)
        .count();
    assert_eq!(skips, 2);
}

#[test]
fn nether_spirit_only_creature_card_at_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Nether Spirit",
        "If Nether Spirit isn't the only creature card in your graveyard as its ability resolves, it remains in your graveyard."
    );
    ruling!(
        "Nether Spirit",
        "If Nether Spirit isn't the only creature card in your graveyard as your upkeep begins, its ability won't trigger."
    );
    supported("Nether Spirit");
    // Another creature card as upkeep begins: no trigger.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Nether Spirit");
    t.graveyard(P0, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "only creature card"), 0);
    // Alone as it triggers, another creature card arrives in response: it stays.
    let mut t = TestGame::new(2);
    let ns = t.graveyard(P0, "Nether Spirit");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "only creature card"), 1);
    t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(is_in(&t, ns, Zone::Graveyard(P0)));
    // Still alone as it resolves: it returns.
    let mut t = TestGame::new(2);
    let ns = t.graveyard(P0, "Nether Spirit");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(ns));
}

#[test]
fn retriever_phoenix_dying_with_its_learn_trigger_can_return() {
    cr!("614.1a", "701.48a");
    ruling!(
        "Retriever Phoenix",
        "If Retriever Phoenix somehow dies with its triggered ability on the stack, you may return it to the battlefield with that triggered ability."
    );
    supported("Retriever Phoenix");
    let mut t = TestGame::new(2);
    let spell = cast_new(&mut t, P0, "Retriever Phoenix", &[]);
    t.resolve();
    let ph = t.g.current(spell);
    assert!(t.on_battlefield(ph));
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, ph);
    assert!(t.in_graveyard(P0, "Retriever Phoenix"));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(ph));
}

#[test]
fn silversmote_ghoul_intervening_if_at_end_step() {
    cr!("603.4", "513.1a");
    ruling!(
        "Silversmote Ghoul",
        "If Silversmote Ghoul isn't in your graveyard as your end step begins, its first ability won't trigger at all."
    );
    ruling!(
        "Silversmote Ghoul",
        "If you haven't gained 3 life by the time your end step begins, Silversmote Ghoul's first ability won't trigger at all."
    );
    supported("Silversmote Ghoul");
    // Life gained, but the Ghoul is put into the graveyard during the end step.
    let mut t = TestGame::new(2);
    let g = t.hand(P0, "Silversmote Ghoul");
    cast_new(&mut t, P0, "Healing Salve", &[Entity::Player(P0)]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    let g = move_to(&mut t, g, Zone::Graveyard(P0)).unwrap();
    assert_eq!(triggers_on_stack(&t, "gained 3 or more life"), 0);
    t.resolve_all();
    assert!(is_in(&t, g, Zone::Graveyard(P0)));
    // In the graveyard, but the life is gained only during the end step.
    let mut t = TestGame::new(2);
    let g = t.graveyard(P0, "Silversmote Ghoul");
    t.advance_to(P0, Step::End);
    assert_eq!(triggers_on_stack(&t, "gained 3 or more life"), 0);
    cast_new(&mut t, P0, "Healing Salve", &[Entity::Player(P0)]);
    t.resolve_all();
    assert!(is_in(&t, g, Zone::Graveyard(P0)));
    // Both: it returns.
    let mut t = TestGame::new(2);
    let g = t.graveyard(P0, "Silversmote Ghoul");
    cast_new(&mut t, P0, "Healing Salve", &[Entity::Player(P0)]);
    t.resolve_all();
    end_step(&mut t, P0);
    assert!(t.on_battlefield(g));
}

/// Sword of the Meek in P0's graveyard.
fn sword() -> (TestGame, ObjectId) {
    supported("Sword of the Meek");
    let mut t = TestGame::new(2);
    let s = t.graveyard(P0, "Sword of the Meek");
    (t, s)
}

#[test]
fn sword_of_the_meek_returns_unattached_if_the_creature_left() {
    cr!("701.3b", "608.2b");
    ruling!(
        "Sword of the Meek",
        "If Sword of the Meek can't be attached to the creature that caused its last ability to trigger, most likely because that creature has left the battlefield, Sword of the Meek returns to the battlefield and remains unattached."
    );
    let (mut t, s) = sword();
    let m = t.enter(P0, "Memnite");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, m);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(s));
    assert_eq!(attached_to(&t, s), None);
}

#[test]
fn sword_of_the_meek_checks_static_abilities_as_the_creature_enters() {
    cr!("603.6a", "603.10");
    ruling!(
        "Sword of the Meek",
        "If a creature is entering the battlefield under your control, consider static abilities to determine whether its power and toughness are both 1."
    );
    // Glorious Anthem makes the 1/1 a 2/2: no trigger.
    let (mut t, _s) = sword();
    t.battlefield(P0, "Glorious Anthem");
    t.enter(P0, "Memnite");
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn sword_of_the_meek_attaches_even_if_the_creature_grew() {
    cr!("603.4", "608.2b");
    ruling!(
        "Sword of the Meek",
        "Once the last ability has triggered, changing the power or toughness of the creature won't stop you from returning Sword of the Meek and attaching it to the creature."
    );
    let (mut t, s) = sword();
    let m = t.enter(P0, "Memnite");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_p206_common::pump(&mut t, m, 3, 3);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(attached_to(&t, s), Some(obj(m)));
}

#[test]
fn syrix_must_already_be_in_the_graveyard() {
    cr!("603.10a", "603.2");
    ruling!(
        "Syrix, Carrier of the Flame",
        "If Syrix dies at the same time as another Phoenix, its last ability won't trigger."
    );
    supported("Syrix, Carrier of the Flame");
    let mut t = TestGame::new(2);
    let syrix = t.battlefield(P0, "Syrix, Carrier of the Flame");
    let ph = t.battlefield(P0, "Warcry Phoenix");
    t.g.destroy_all(vec![syrix, ph], None, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "cast ~"), 0);
    // Already in the graveyard: another Phoenix dying triggers it.
    let ph2 = t.battlefield(P0, "Warcry Phoenix");
    destroy(&mut t, ph2);
    assert_eq!(triggers_on_stack(&t, "cast ~"), 1);
}

#[test]
fn geralfs_masterpiece_survives_zero_toughness_mid_resolution() {
    cr!("704.3", "608.2c");
    ruling!(
        "Geralf's Masterpiece",
        "As long as it has toughness 1 or greater again after the spell or ability resolves, it will survive."
    );
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Geralf's Masterpiece");
    t.g.players[0].hand.clear();
    for _ in 0..5 {
        t.hand(P0, "Forest");
    }
    // Faithless Looting: 5 cards in hand after casting, 7 after drawing (a 0/0), 5 after
    // discarding.
    cast_new(&mut t, P0, "Faithless Looting", &[]);
    assert_eq!(t.pt(gm).1, 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 5);
    assert!(t.on_battlefield(gm));
}

#[test]
fn polukranos_greater_damage_removes_all_counters() {
    cr!("615.1a", "704.5f");
    ruling!(
        "Polukranos, Unchained",
        "If an amount of damage would be dealt to Polukranos greater than the number of +1/+1 counters on it, all of that damage is prevented, and all of those counters are removed."
    );
    supported("Polukranos, Unchained");
    let mut t = TestGame::new(2);
    let pk = t.enter(P0, "Polukranos, Unchained");
    assert_eq!(t.counters(pk, counters::PLUS1), 6);
    // Indomitable Will (+1/+2) keeps it alive at 1/2 once the counters are gone, so the
    // 2 damage beyond the counters would be lethal if it weren't prevented.
    attach_new(&mut t, P0, "Indomitable Will", pk);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 8, pk);
    assert!(t.on_battlefield(pk));
    assert_eq!(t.counters(pk, counters::PLUS1), 0);
    assert_eq!(t.obj_now(pk).damage, 0);
    assert_eq!(t.pt(pk), (1, 2));
}

#[test]
fn polukranos_unpreventable_damage_and_counter_removal_together() {
    cr!("615.12", "704.3");
    ruling!(
        "Polukranos, Unchained",
        "If damage that can't be prevented is dealt to Polukranos, the damage is dealt and you'll simultaneously remove that many +1/+1 counters from it, all before the game checks for lethal damage."
    );
    let mut t = TestGame::new(2);
    let pk = t.enter(P0, "Polukranos, Unchained");
    cast_new(&mut t, P1, "Flaring Pain", &[]);
    t.resolve_all();
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 2, pk);
    // 2 damage marked on a 4/4.
    assert!(t.on_battlefield(pk));
    assert_eq!(t.counters(pk, counters::PLUS1), 4);
    assert_eq!(t.obj_now(pk).damage, 2);
    // 3 more: 5 damage on a 1/1.
    damage(&mut t, src, 3, pk);
    assert!(t.in_graveyard(P0, "Polukranos, Unchained"));
}

#[test]
fn polukranos_fight_with_an_illegal_target_or_missing_polukranos() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Polukranos, Unchained",
        "If the target creature is an illegal target when Polukranos's activated ability tries to resolve, the ability doesn't resolve. If Polukranos is no longer on the battlefield, the target creature won't deal or be dealt damage."
    );
    // Polukranos leaves: the target isn't dealt damage.
    let mut t = TestGame::new(2);
    let pk = t.enter(P0, "Polukranos, Unchained");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, pk, "fights").unwrap();
    bounce(&mut t, pk);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(t.on_battlefield(bears));
    // The target is gone: Polukranos isn't dealt damage either.
    let mut t = TestGame::new(2);
    let pk = t.enter(P0, "Polukranos, Unchained");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, pk, "fights").unwrap();
    bounce(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(pk, counters::PLUS1), 6);
}

#[test]
fn lightning_phoenix_counts_damage_to_a_player_who_lost() {
    cr!("603.4", "800.4a");
    ruling!(
        "Lightning Phoenix",
        "If an opponent is dealt 3 or more damage and loses the game, Lightning Phoenix's ability triggers as your end step begins."
    );
    supported("Lightning Phoenix");
    let mut t = TestGame::new(3);
    let lp = t.graveyard(P0, "Lightning Phoenix");
    t.lands(P0, "Mountain", 2);
    t.g.players[1].life = 3;
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.has_lost(P1));
    t.answer_yes(P0, true);
    end_step(&mut t, P0);
    assert!(t.on_battlefield(lp));
}

#[test]
fn lightning_phoenix_counts_earlier_damage_and_ignores_life_gain() {
    cr!("603.4", "120.3");
    ruling!(
        "Lightning Phoenix",
        "Lightning Phoenix's last ability looks at how much damage was dealt to opponents in the turn, even if it wasn't in your graveyard when the damage was dealt."
    );
    ruling!(
        "Lightning Phoenix",
        "Lightning Phoenix's last ability triggers only if it's in your graveyard as your end step begins and only if an opponent was dealt 3 or more damage before your end step begins."
    );
    // Damage dealt while it's on the battlefield, then the opponent gains life; it dies.
    let mut t = TestGame::new(2);
    let lp = t.battlefield(P0, "Lightning Phoenix");
    t.lands(P0, "Mountain", 2);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    cast_new(&mut t, P1, "Healing Salve", &[Entity::Player(P1)]);
    t.resolve_all();
    destroy(&mut t, lp);
    t.answer_yes(P0, true);
    end_step(&mut t, P0);
    assert!(t.on_battlefield(lp));
    // Damage dealt only during the end step: no trigger.
    let mut t = TestGame::new(2);
    let lp = t.graveyard(P0, "Lightning Phoenix");
    t.lands(P0, "Mountain", 2);
    t.advance_to(P0, Step::End);
    assert_eq!(triggers_on_stack(&t, "dealt 3 or more damage"), 0);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(is_in(&t, lp, Zone::Graveyard(P0)));
}

#[test]
fn ichorid_cant_share_or_return_an_exiled_ichorid() {
    cr!("601.2h", "400.7");
    ruling!(
        "Ichorid",
        "If multiple Ichorids are in your graveyard, you can't exile the same creature card to return each of them to the battlefield."
    );
    ruling!(
        "Ichorid",
        "If one Ichorid's ability exiles another, the exiled Ichorid's ability can't return it to the battlefield."
    );
    supported("Ichorid");
    // Two Ichorids, one other black creature card: only one returns.
    let mut t = TestGame::new(2);
    let i1 = t.graveyard(P0, "Ichorid");
    let i2 = t.graveyard(P0, "Ichorid");
    let traitor = t.graveyard(P0, "Nether Traitor");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(traitor)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let back = [i1, i2].iter().filter(|i| t.on_battlefield(**i)).count();
    assert_eq!(back, 1);
    assert!(t.in_exile("Nether Traitor"));
    // Two Ichorids alone: the first exiles the other, which can't return.
    let mut t = TestGame::new(2);
    let i1 = t.graveyard(P0, "Ichorid");
    let i2 = t.graveyard(P0, "Ichorid");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let back = [i1, i2].iter().filter(|i| t.on_battlefield(**i)).count();
    let exiled = [i1, i2]
        .iter()
        .filter(|i| is_in(&t, **i, Zone::Exile))
        .count();
    assert_eq!((back, exiled), (1, 1));
}

#[test]
fn skeleton_crew_triggers_once_per_leaving_event() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Skeleton Crew",
        "If two or more creature cards leave your graveyard as a single event, Skeleton Crew's second ability will trigger once. If they leave your graveyard as separate events, it will trigger once for each event."
    );
    supported("Skeleton Crew");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skeleton Crew");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    t.g.turn.priority = Some(P1);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    activate_containing(&mut t, P1, crypt, "Exile").unwrap();
    t.resolve_all();
    assert!(is_in(&t, a, Zone::Exile) && is_in(&t, b, Zone::Exile));
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Skeleton"), 1);
    let c = t.graveyard(P0, "Grizzly Bears");
    let d = t.graveyard(P0, "Hill Giant");
    exile_now(&mut t, c);
    exile_now(&mut t, d);
    t.resolve_all();
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Skeleton"), 3);
}

#[test]
fn return_attached_to_a_target_that_became_illegal() {
    cr!("608.2b");
    ruling!(
        "Gryff's Boon",
        "If you activate Gryff's Boon's last ability and the target creature becomes an illegal target in response, the ability doesn't resolve and Gryff's Boon remains in your graveyard."
    );
    ruling!(
        "Eagle's Rescue",
        "If you activate the last ability of Eagle's Rescue and the target creature becomes an illegal target in response, the ability doesn't resolve and Eagle's Rescue remains in your graveyard."
    );
    for (card, lands) in [("Gryff's Boon", "Plains"), ("Eagle's Rescue", "Plains")] {
        supported(card);
        let mut t = TestGame::new(2);
        let aura = t.graveyard(P0, card);
        t.lands(P0, lands, 4);
        let target = t.battlefield(P0, "Raging Goblin");
        t.answer_targets(P0, &[obj(target)]);
        activate_containing(&mut t, P0, aura, "Return").unwrap();
        bounce(&mut t, target);
        t.resolve_all();
        assert!(is_in(&t, aura, Zone::Graveyard(P0)), "{card}");
        // A legal target: it returns attached.
        let target = t.battlefield(P0, "Raging Goblin");
        t.lands(P0, lands, 4);
        t.answer_targets(P0, &[obj(target)]);
        activate_containing(&mut t, P0, aura, "Return").unwrap();
        t.resolve_all();
        assert_eq!(attached_to(&t, aura), Some(obj(target)), "{card}");
    }
}

#[test]
fn nether_shadow_return_is_an_ability_not_a_spell() {
    cr!("113.3c", "701.6a");
    ruling!(
        "Nether Shadow",
        "It can't be countered with something that counters spells."
    );
    supported("Nether Shadow");
    let mut t = TestGame::new(2);
    let ns = t.graveyard(P0, "Nether Shadow");
    // Three creature cards above it.
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    lands_for_cost(&mut t, P1, "Counterspell");
    let cs = t.hand(P1, "Counterspell");
    let top = t.g.stack[0];
    assert!(t.cast(P1, cs).target(top).try_go().is_err());
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(ns));
}

#[test]
fn flamewake_phoenix_rechecks_its_condition_on_resolution() {
    cr!("603.4");
    ruling!(
        "Flamewake Phoenix",
        "If it does trigger, the ability will check again as it tries to resolve. If you don't control a creature with power 4 or greater at that time, the ability won't resolve."
    );
    supported("Flamewake Phoenix");
    // No creature with power 4: no trigger.
    let mut t = TestGame::new(2);
    let fp = t.graveyard(P0, "Flamewake Phoenix");
    t.lands(P0, "Mountain", 1);
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // It triggers; the big creature leaves in response: no payment asked, no return.
    let mut t = TestGame::new(2);
    let fp2 = t.graveyard(P0, "Flamewake Phoenix");
    t.lands(P0, "Mountain", 1);
    let giant = t.battlefield(P0, "Craw Wurm");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    bounce(&mut t, giant);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, P0, from), 0);
    assert!(is_in(&t, fp2, Zone::Graveyard(P0)));
    let _ = fp;
}

#[test]
fn attacks_each_combat_creatures_that_cant_attack_dont() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Flamewake Phoenix",
        "If Flamewake Phoenix can't attack for any reason (such as being tapped), then it doesn't attack. If there's a cost associated with having it attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."
    );
    ruling!(
        "Underworld Rage-Hound",
        "If Underworld Rage-Hound can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack."
    );
    ruling!(
        "Akoum Firebird",
        "If, during its controller's declare attackers step, Akoum Firebird is tapped or is affected by a spell or ability that says it can't attack, then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost."
    );
    for card in [
        "Flamewake Phoenix",
        "Underworld Rage-Hound",
        "Akoum Firebird",
    ] {
        supported(card);
        // Tapped: declaring no attackers is legal.
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, card);
        tap(&mut t, c);
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(crate::r_s09_common::legal_attack(&mut t, &[]), "{card} tapped");
        // A cost to attack (Ghostly Prison): not forced to pay, so no attack is legal.
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        t.battlefield(P1, "Ghostly Prison");
        t.lands(P0, "Plains", 2);
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(crate::r_s09_common::legal_attack(&mut t, &[]), "{card} with a cost");
        // Untapped with no cost: it must attack.
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!crate::r_s09_common::legal_attack(&mut t, &[]), "{card} must attack");
    }
    // Rage-Hound that came under P0's control this turn (without haste) can't attack.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P1, "Underworld Rage-Hound");
    crate::r_s06_common::give_control(&mut t, h, P0);
    assert_eq!(t.obj_now(h).controller, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s09_common::legal_attack(&mut t, &[]));
}

#[test]
fn forsaken_miner_must_be_in_the_graveyard_as_the_crime_is_committed() {
    cr!("700.13", "603.2");
    ruling!(
        "Forsaken Miner",
        "However, if you cast a spell with an additional cost of \"Sacrifice a creature\" and you sacrifice Forsaken Miner to pay that cost, Forsaken Miner's ability will trigger as long as you committed a crime with that spell."
    );
    ruling!(
        "Forsaken Miner",
        "Forsaken Miner's last ability triggers only if Forsaken Miner is in your graveyard at the moment you cast a spell, activate an ability, or put a triggered ability on the stack with one or more targets that constitute a crime."
    );
    supported("Forsaken Miner");
    // On the battlefield as the crime is committed: no trigger.
    let mut t = TestGame::new(2);
    let fm = t.battlefield(P0, "Forsaken Miner");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "commit a crime"), 0);
    t.resolve_all();
    destroy(&mut t, fm);
    assert_eq!(triggers_on_stack(&t, "commit a crime"), 0);
    // Sacrificed to pay for a spell targeting an opponent's creature: it triggers.
    let mut t = TestGame::new(2);
    let fm = t.battlefield(P0, "Forsaken Miner");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[obj(fm)]);
    cast_new(&mut t, P0, "Bone Splinters", &[obj(bears)]);
    t.settle();
    assert!(t.in_graveyard(P0, "Forsaken Miner"));
    assert_eq!(triggers_on_stack(&t, "commit a crime"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(fm));
}

#[test]
fn nethergoyf_counts_card_types_not_cards() {
    cr!("604.3", "205.2a");
    ruling!(
        "Nethergoyf",
        "Nethergoyf's first ability counts card types, not cards. If the only card in your graveyard is a single artifact creature card, Nethergoyf will be a 2/3."
    );
    supported("Nethergoyf");
    let mut t = TestGame::new(2);
    let ng = t.battlefield(P0, "Nethergoyf");
    t.g.players[0].graveyard.clear();
    t.graveyard(P0, "Ornithopter");
    t.g.recompute();
    assert_eq!(t.pt(ng), (2, 3));
    // Ten artifact cards and ten creature cards: still two card types.
    for _ in 0..10 {
        t.graveyard(P0, "Sol Ring");
        t.graveyard(P0, "Grizzly Bears");
    }
    t.g.recompute();
    assert_eq!(t.pt(ng), (2, 3));
}
