//! Rulings batch P039 — Vehicles (crew): what they do when they attack, enter or leave
//! the battlefield, counters on noncreature Vehicles, Vehicles that become creatures by
//! other effects, and the historic cards grouped with them.

use crate::r_p039_common::*;
use crate::r_p039_creatureland::{activate_text, can_activate_text};
use crate::r_s01_common::{attack_with, block_and_finish, supported, triggers_on_stack, watch};
use crate::r_s02_common::{can_activate, can_cast, can_play_land, destroy, target_candidates};
use crate::r_s04_common::crew;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Crews `vehicle` with `crew` (asserting it worked) and resolves the crew ability and
/// any "becomes crewed" triggers.
fn crewed(t: &mut TestGame, vehicle: ObjectId, by: &[ObjectId]) {
    assert!(crew(t, P0, vehicle, by), "couldn't crew");
    t.resolve_all();
    assert!(creature_now(t, vehicle));
}

/// Returns a permanent to its owner's hand (as a spell or ability would) and settles.
fn bounce(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let owner = t.g.obj(id).owner;
    t.g.move_object(id, Zone::Hand(owner), mtg_engine::events::MoveCause::Effect, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

// ---------------------------------------------------------------------------
// Attack triggers
// ---------------------------------------------------------------------------

#[test]
fn mobile_garrison_untapping_a_creature_doesnt_let_it_attack() {
    cr!("508.1", "508.1a");
    ruling!(
        "Mobile Garrison",
        "All attackers are chosen at once. You can't attack with Mobile Garrison, untap a tapped creature, and then attack with that creature."
    );
    supported("Mobile Garrison");
    let mut t = TestGame::new(2);
    let g = old(&mut t, P0, "Mobile Garrison");
    let bears = old(&mut t, P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(&mut t, g, &[bears]);
    assert!(t.obj_now(bears).tapped);
    t.answer_targets(P0, &[o(bears)]);
    attack_with(&mut t, &[(g, pl(P1))]);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(t.g.is_attacking(g));
    assert!(!t.g.is_attacking(bears));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 17);
}

/// "Whenever a creature you control attacks" triggers (Hellrider) on the stack.
fn hellrider_triggers(t: &TestGame) -> usize {
    triggers_on_stack(t, "deals 1 damage to the player or planeswalker it's attacking")
}

#[test]
fn leonin_warleader_tokens_were_never_declared_as_attackers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Leonin Warleader",
        "Although the tokens are attacking, they were never declared as attacking creatures (for the purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellrider");
    let w = old(&mut t, P0, "Leonin Warleader");
    attack_with(&mut t, &[(w, pl(P1))]);
    assert_eq!(hellrider_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(named(&t, P0, "Cat Token").len(), 2);
    assert!(named(&t, P0, "Cat Token").iter().all(|c| t.g.is_attacking(*c)));
    assert_eq!(t.life(P1), 19);
    assert_eq!(hellrider_triggers(&t), 0);
}

#[test]
fn parhelion_ii_tokens_were_never_declared_as_attackers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Parhelion II",
        "Although the tokens are attacking, they were never declared as attacking creatures (for the purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    supported("Parhelion II");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellrider");
    let p = old(&mut t, P0, "Parhelion II");
    let a = old(&mut t, P0, "Hill Giant");
    let b = old(&mut t, P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(&mut t, p, &[a, b]);
    attack_with(&mut t, &[(p, pl(P1))]);
    assert_eq!(hellrider_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(named(&t, P0, "Angel Token").len(), 2);
    assert_eq!(hellrider_triggers(&t), 0);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn parhelion_ii_tokens_can_attack_different_players_and_planeswalkers() {
    cr!("508.4", "506.2");
    ruling!(
        "Parhelion II",
        "You choose which players or planeswalkers the two tokens are attacking. They don't have to be attacking the same player or planeswalker that Parhelion II is attacking"
    );
    let mut t = TestGame::new(3);
    let jace = t.battlefield(P1, "Jace Beleren");
    let p = old(&mut t, P0, "Parhelion II");
    let a = old(&mut t, P0, "Hill Giant");
    let b = old(&mut t, P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(&mut t, p, &[a, b]);
    attack_with(&mut t, &[(p, pl(P1))]);
    t.answer_choose(P0, &[pl(P2)]);
    t.answer_choose(P0, &[o(jace)]);
    let from = mark(&t);
    t.resolve_all();
    // P0 was asked, once per token, with every player and planeswalker it could attack.
    let asks: Vec<(PlayerId, Vec<Entity>)> = t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("attacking") => Some((*q, candidates.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(asks.len(), 2);
    for (q, c) in &asks {
        assert_eq!(*q, P0);
        assert!(c.contains(&pl(P1)) && c.contains(&pl(P2)) && c.contains(&o(jace)));
    }
    block_and_finish(&mut t, P1, &[]);
    // Parhelion II (5) at P1, an Angel (4) at P2, an Angel at Jace (loyalty 3).
    assert_eq!((t.life(P1), t.life(P2)), (15, 16));
    assert!(!t.on_battlefield(jace));
}

#[test]
fn untethered_express_keeps_its_counters_when_it_stops_being_a_creature() {
    cr!("122.1", "301.7b");
    ruling!(
        "Untethered Express",
        "Any +1/+1 counters on Untethered Express remain on it when it stops being a creature."
    );
    let mut t = TestGame::new(2);
    let x = old(&mut t, P0, "Untethered Express");
    let elf = old(&mut t, P0, "Llanowar Elves");
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(&mut t, x, &[elf]);
    attack_with(&mut t, &[(x, pl(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);
    t.advance_to(P1, Step::Upkeep);
    assert!(!creature_now(&t, x));
    assert_eq!(t.counters(x, counters::PLUS1), 1);
    t.advance_to(P0, Step::PrecombatMain);
    crewed(&mut t, x, &[elf]);
    assert_eq!(t.pt(x), (5, 5));
}

#[test]
fn ballista_charger_kills_a_would_be_blocker_before_blocks() {
    cr!("508.2", "509.1a", "704.5g");
    ruling!(
        "Ballista Charger",
        "Ballista Charger's triggered ability resolves before blockers are chosen. A creature dealt lethal damage this way won't be around to block."
    );
    supported("Ballista Charger");
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Ballista Charger");
    let g = old(&mut t, P0, "Hill Giant");
    let elf = old(&mut t, P1, "Llanowar Elves");
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(&mut t, b, &[g]);
    t.answer_targets(P0, &[o(elf)]);
    attack_with(&mut t, &[(b, pl(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    let from = mark(&t);
    block_and_finish(&mut t, P1, &[]);
    for (_, d) in &t.asked()[from..] {
        if let Decision::DeclareBlockers { options } = d {
            assert!(options.is_empty(), "{options:?}");
        }
    }
    assert_eq!(t.life(P1), 14);
}

#[test]
fn esikas_chariot_token_copy_has_its_enters_abilities() {
    cr!("707.2", "111.4", "614.1c");
    ruling!(
        "Esika's Chariot",
        "Any enters-the-battlefield abilities of the token you create will trigger when it enters the battlefield. Any \"As [this permanent] enters the battlefield\" or \"[This permanent] enters the battlefield with\" abilities of the copied token will also work."
    );
    for (name, check_draw) in [("Wall of Omens", true), ("Kalonian Hydra", false)] {
        let mut t = TestGame::new(2);
        let ch = old(&mut t, P0, "Esika's Chariot");
        let orig = old(&mut t, P0, name);
        if !check_draw {
            // (Put onto the battlefield directly, it didn't get its four counters.)
            t.g.add_counters(o(orig), counters::PLUS1, 4, None);
        }
        let giant = old(&mut t, P0, "Hill Giant");
        let elf = old(&mut t, P0, "Llanowar Elves");
        // Cackling Counterpart: "Create a token that's a copy of target creature you
        // control."
        cast_named(&mut t, P0, "Cackling Counterpart", &[o(orig)]);
        t.resolve_all();
        let tok = *named(&t, P0, name)
            .iter()
            .find(|x| t.obj_now(**x).is_token())
            .unwrap();
        t.g.objects[tok.0 as usize].summoning_sick = false;
        t.set_step(P0, Step::BeginningOfCombat);
        crewed(&mut t, ch, &[giant, elf]);
        let hand = t.hand_size(P0);
        t.answer_targets(P0, &[o(tok)]);
        attack_with(&mut t, &[(ch, pl(P1))]);
        t.resolve_all();
        let toks: Vec<ObjectId> = named(&t, P0, name)
            .into_iter()
            .filter(|x| t.obj_now(*x).is_token())
            .collect();
        assert_eq!(toks.len(), 2);
        if check_draw {
            assert_eq!(t.hand_size(P0), hand + 1);
        } else {
            for x in toks {
                assert_eq!(t.counters(x, counters::PLUS1), 4);
            }
        }
    }
}

#[test]
fn esikas_chariot_copy_doesnt_copy_status_counters_or_attacking() {
    cr!("707.2", "110.5", "508.4");
    ruling!(
        "Esika's Chariot",
        "The token you create copies the original characteristics of the token it's copying as stated by the effect that created that token. The newly created token doesn't copy whether the original token is tapped or untapped, whether it has any counters on it"
    );
    let mut t = TestGame::new(2);
    let ch = enter(&mut t, P0, "Esika's Chariot");
    t.resolve_all();
    let cats = named(&t, P0, "Cat Token");
    assert_eq!(cats.len(), 2);
    for c in &cats {
        t.g.objects[c.0 as usize].summoning_sick = false;
    }
    t.g.objects[ch.0 as usize].summoning_sick = false;
    t.g.add_counters(o(cats[0]), counters::PLUS1, 1, None);
    // Giant Growth on the Cat: a non-copy effect.
    cast_named(&mut t, P0, "Giant Growth", &[o(cats[0])]);
    t.resolve_all();
    let giant = old(&mut t, P0, "Hill Giant");
    let elf = old(&mut t, P0, "Llanowar Elves");
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(&mut t, ch, &[giant, elf]);
    t.answer_targets(P0, &[o(cats[0])]);
    attack_with(&mut t, &[(ch, pl(P1)), (cats[0], pl(P1))]);
    t.resolve_all();
    let new: Vec<ObjectId> = named(&t, P0, "Cat Token")
        .into_iter()
        .filter(|c| !cats.contains(c))
        .collect();
    assert_eq!(new.len(), 1);
    let n = t.obj_now(new[0]);
    assert!(!n.tapped);
    assert_eq!(n.counters.values().sum::<u32>(), 0);
    assert!(!t.g.is_attacking(new[0]));
    assert_eq!(t.pt(new[0]), (2, 2));
    assert!(t.g.is_attacking(cats[0]));
}

#[test]
fn golden_argosy_returns_the_crew_even_if_it_left() {
    cr!("603.7c", "610.3");
    ruling!(
        "Golden Argosy",
        "The exiled creatures will be returned at the beginning of the next end step even if Golden Argosy is no longer on the battlefield at that time."
    );
    let mut t = TestGame::new(2);
    let a = old(&mut t, P0, "Golden Argosy");
    let b = old(&mut t, P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(&mut t, a, &[b]);
    attack_with(&mut t, &[(a, pl(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    destroy(&mut t, a);
    assert!(!t.on_battlefield(a));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = named(&t, P0, "Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert!(t.obj_now(back[0]).tapped);
}

#[test]
fn reaver_titan_protection_sees_x_in_the_spells_mana_value() {
    cr!("202.3e", "702.16b");
    ruling!(
        "Reaver Titan",
        "While a spell with X in its cost is on the stack, its mana value takes the chosen value of X into account."
    );
    supported("Reaver Titan");
    for x in [2i64, 3] {
        let mut t = TestGame::new(2);
        let r = old(&mut t, P1, "Reaver Titan");
        let a = old(&mut t, P1, "Hill Giant");
        let b = old(&mut t, P1, "Hill Giant");
        // P1 crews it in P0's main phase.
        assert!(crew(&mut t, P1, r, &[a, b]));
        t.resolve_all();
        assert!(creature_now(&t, r));
        // Blaze: "{X}{R}: Blaze deals X damage to any target."
        mana_all(&mut t, P0, 4);
        let c = t.hand(P0, "Blaze");
        let from = mark(&t);
        t.answer_targets(P0, &[pl(P1)]);
        t.cast(P0, c).x(x).go();
        let cands = target_candidates(&t, P0, from);
        assert_eq!(cands.len(), 1);
        // X = 2: mana value 3, Reaver Titan can't be targeted; X = 3: mana value 4.
        assert_eq!(cands[0].contains(&o(r)), x == 3, "x={x}");
        assert!(cands[0].contains(&o(a)));
    }
}

#[test]
fn recon_craft_theta_proliferates_permanents_and_players_only() {
    cr!("701.34a");
    ruling!(
        "Recon Craft Theta",
        "You can choose any permanent that has a counter, including ones controlled by opponents. You can choose any player who has a counter, including opponents. You can't choose cards in any zone other than the battlefield"
    );
    supported("Recon Craft Theta");
    let mut t = TestGame::new(2);
    let r = old(&mut t, P0, "Recon Craft Theta");
    let b = old(&mut t, P0, "Grizzly Bears");
    let foe = old(&mut t, P1, "Hill Giant");
    t.g.add_counters(o(foe), counters::PLUS1, 1, None);
    t.g.add_counters(pl(P1), counters::POISON, 1, None);
    // A card with a counter in exile (suspended) and in a graveyard.
    let susp = t.exile(P0, "Grizzly Bears");
    t.g.add_counters(o(susp), counters::TIME, 2, None);
    let dead = t.graveyard(P1, "Hill Giant");
    t.g.add_counters(o(dead), counters::PLUS1, 1, None);
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(&mut t, r, &[b]);
    t.answer_choose(P0, &[o(foe), pl(P1)]);
    let from = mark(&t);
    attack_with(&mut t, &[(r, pl(P1))]);
    t.resolve_all();
    let cands: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(cands.len(), 1, "{cands:?}");
    assert!(cands[0].contains(&o(foe)) && cands[0].contains(&pl(P1)));
    assert!(!cands[0].contains(&o(susp)) && !cands[0].contains(&o(dead)));
    assert_eq!(t.counters(foe, counters::PLUS1), 2);
    assert_eq!(t.g.player(P1).counters.get(counters::POISON).copied(), Some(2));
    assert_eq!(t.counters(susp, counters::TIME), 2);
}

// ---------------------------------------------------------------------------
// Enters / leaves the battlefield
// ---------------------------------------------------------------------------

#[test]
fn foreboding_steamboat_leaving_first_exiles_nothing() {
    cr!("610.3c");
    ruling!(
        "Foreboding Steamboat",
        "If Foreboding Steamboat leaves the battlefield before its first triggered ability resolves, the chosen creatures won't be exiled at all."
    );
    supported("Foreboding Steamboat");
    let mut t = TestGame::new(2);
    old(&mut t, P0, "Grizzly Bears");
    old(&mut t, P0, "Hill Giant");
    old(&mut t, P1, "Grizzly Bears");
    old(&mut t, P1, "Hill Giant");
    let s = enter(&mut t, P0, "Foreboding Steamboat");
    destroy(&mut t, s);
    t.resolve_all();
    assert!(!t.in_exile("Grizzly Bears") && !t.in_exile("Hill Giant"));
    assert_eq!(t.g.permanents().filter(|o| o.is_creature()).count(), 4);
}

#[test]
fn foreboding_steamboat_choices_in_turn_order_then_exiled_at_once() {
    cr!("101.4", "610.3");
    ruling!(
        "Foreboding Steamboat",
        "Starting with the player whose turn it is, each player in turn order chooses two nontoken, non-Vehicle creatures they control, then all the creatures chosen by all players are exiled at the same time."
    );
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let mut mine = vec![];
    for p in [P0, P1, P2] {
        let a = old(&mut t, p, "Grizzly Bears");
        let b = old(&mut t, p, "Hill Giant");
        old(&mut t, p, "Llanowar Elves");
        mine.push((p, a, b));
    }
    // A token and a Vehicle P1 controls can't be chosen.
    let tok = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    let v = old(&mut t, P1, "Untethered Express");
    for (p, a, b) in &mine {
        t.answer_choose(*p, &[o(*a), o(*b)]);
    }
    // The number of creatures on the battlefield whenever a card is exiled.
    let seen: crate::r_s01_common::Seen<usize> = Default::default();
    let s2 = seen.clone();
    t.g.observer = Some(mtg_engine::game::EventObserver::new(move |g, ev| {
        if let mtg_engine::events::Event::ZoneChange { to: Zone::Exile, .. } = ev {
            s2.lock()
                .unwrap()
                .push(g.permanents().filter(|o| o.is_creature()).count());
        }
    }));
    let from = mark(&t);
    // P1 (the active player) controls the Steamboat.
    let id = new_card(&mut t, P1, "Foreboding Steamboat");
    enter_obj(&mut t, P1, id);
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } => {
                assert!(!candidates.contains(&o(tok)) && !candidates.contains(&o(v)));
                Some(*p)
            }
            _ => None,
        })
        .collect();
    assert_eq!(order, vec![P1, P2, P0]);
    for (_, a, b) in &mine {
        assert_eq!(t.zone(*a), Zone::Exile);
        assert_eq!(t.zone(*b), Zone::Exile);
    }
    // Six exiled; every zone change saw all of them gone already (3 Elves and the
    // token left).
    let s = seen.lock().unwrap().clone();
    assert_eq!(s, vec![4; 6]);
}

#[test]
fn nautiloid_ship_returned_is_a_new_object() {
    cr!("400.7", "607.2a");
    ruling!(
        "Nautiloid Ship",
        "If Nautiloid Ship leaves the battlefield and later returns to the battlefield, it is a new object with no memory of its previous existence."
    );
    supported("Nautiloid Ship");
    for bounced in [false, true] {
        let mut t = TestGame::new(2);
        t.graveyard(P1, "Hill Giant");
        t.answer_targets(P0, &[pl(P1)]);
        let mut s = enter(&mut t, P0, "Nautiloid Ship");
        t.resolve_all();
        assert!(t.in_exile("Hill Giant"));
        if bounced {
            bounce(&mut t, s);
            t.answer_targets(P0, &[pl(P1)]);
            s = enter(&mut t, P0, "Nautiloid Ship");
            t.resolve_all();
        }
        t.g.objects[s.0 as usize].summoning_sick = false;
        let a = old(&mut t, P0, "Hill Giant");
        t.set_step(P0, Step::BeginningOfCombat);
        crewed(&mut t, s, &[a]);
        yes(&mut t, P0, true);
        let giant = t.g.find_in_zone(Zone::Exile, "Hill Giant")[0];
        t.answer_choose(P0, &[o(giant)]);
        attack_with(&mut t, &[(s, pl(P1))]);
        block_and_finish(&mut t, P1, &[]);
        t.resolve_all();
        assert_eq!(t.life(P1), 15);
        assert_eq!(named(&t, P0, "Hill Giant").len(), 1 + !bounced as usize);
    }
}

#[test]
fn unidentified_hovership_exiles_even_if_it_left_first() {
    cr!("603.2", "608.2b");
    ruling!(
        "Unidentified Hovership",
        "If Unidentified Hovership leaves the battlefield before its second ability resolves, the ability still exiles the target creature."
    );
    let mut t = TestGame::new(2);
    let foe = old(&mut t, P1, "Hill Giant");
    t.answer_targets(P0, &[o(foe)]);
    let h = enter(&mut t, P0, "Unidentified Hovership");
    destroy(&mut t, h);
    t.resolve_all();
    assert_eq!(t.zone(foe), Zone::Exile);
}

/// The face-down permanents `p` controls.
fn face_down(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.face_down)
        .count()
}

#[test]
fn unidentified_hovership_each_owner_of_an_exiled_card_manifests_dread() {
    cr!("603.2", "701.62a");
    ruling!(
        "Unidentified Hovership",
        "If Unidentified Hovership's second ability exiled more than one card (possibly because another effect caused it to trigger an additional time), each player who owns one or more of the exiled cards manifests dread."
    );
    let mut t = TestGame::new(3);
    // Panharmonicon: its enters ability triggers twice.
    t.battlefield(P0, "Panharmonicon");
    let a = old(&mut t, P1, "Hill Giant");
    let b = old(&mut t, P2, "Grizzly Bears");
    t.answer_targets(P0, &[o(a)]);
    t.answer_targets(P0, &[o(b)]);
    let h = enter(&mut t, P0, "Unidentified Hovership");
    t.resolve_all();
    assert_eq!((t.zone(a), t.zone(b)), (Zone::Exile, Zone::Exile));
    destroy(&mut t, h);
    t.resolve_all();
    assert_eq!((face_down(&t, P0), face_down(&t, P1), face_down(&t, P2)), (0, 1, 1));
}

#[test]
fn detention_chariot_leaving_first_exiles_nothing() {
    cr!("610.3c");
    ruling!(
        "Detention Chariot",
        "If this Vehicle leaves the battlefield before its triggered ability resolves, the target will never be exiled."
    );
    let mut t = TestGame::new(2);
    let foe = old(&mut t, P1, "Hill Giant");
    t.answer_targets(P0, &[o(foe)]);
    let c = enter(&mut t, P0, "Detention Chariot");
    destroy(&mut t, c);
    t.resolve_all();
    assert!(t.on_battlefield(foe));
    // Otherwise, it is exiled.
    let mut t = TestGame::new(2);
    let foe = old(&mut t, P1, "Hill Giant");
    t.answer_targets(P0, &[o(foe)]);
    enter(&mut t, P0, "Detention Chariot");
    t.resolve_all();
    assert_eq!(t.zone(foe), Zone::Exile);
}

#[test]
fn the_prydwen_knight_keeps_its_bonus_after_the_artifact_leaves() {
    cr!("611.3a");
    ruling!(
        "The Prydwen, Steel Flagship",
        "Once an artifact enters the battlefield under your control in a given turn, it doesn't matter what happens to it later."
    );
    supported("The Prydwen, Steel Flagship");
    let mut t = TestGame::new(2);
    old(&mut t, P0, "The Prydwen, Steel Flagship");
    let orn = enter(&mut t, P0, "Ornithopter");
    t.resolve_all();
    let k = named(&t, P0, "Human Knight Token");
    assert_eq!(k.len(), 1);
    assert_eq!(t.pt(k[0]), (4, 4));
    destroy(&mut t, orn);
    assert_eq!(t.pt(k[0]), (4, 4));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(k[0]), (2, 2));
}

// ---------------------------------------------------------------------------
// Counters and becoming a creature
// ---------------------------------------------------------------------------

#[test]
fn shadowed_caravel_gets_counters_while_not_a_creature() {
    cr!("122.1", "701.44a");
    ruling!(
        "Shadowed Caravel",
        "Noncreature permanents such as Shadowed Caravel can have +1/+1 counters put on them. Those counters remain on it while it's not a creature, and will apply if it becomes a creature."
    );
    let mut t = TestGame::new(2);
    let c = old(&mut t, P0, "Shadowed Caravel");
    // Merfolk Branchwalker explores: a nonland card on top.
    t.library_top(P0, "Grizzly Bears");
    let m = enter(&mut t, P0, "Merfolk Branchwalker");
    t.resolve_all();
    assert!(!creature_now(&t, c));
    assert_eq!(t.counters(c, counters::PLUS1), 1);
    let m = t.g.current(m);
    t.g.objects[m.0 as usize].summoning_sick = false;
    crewed(&mut t, c, &[m]);
    assert_eq!(t.pt(c), (3, 3));
}

#[test]
fn vehicle_animated_by_other_effects_has_its_printed_power_and_toughness() {
    cr!("301.7b", "208.3");
    ruling!(
        "Mech Hangar",
        "If an effect that causes a Vehicle to become a creature doesn't specify its power and toughness, it has the same power and toughness that it would normally have if it were crewed."
    );
    ruling!(
        "Mobilizer Mech",
        "If an effect that causes a Vehicle to become a creature doesn't specify its power and toughness, it has the same power and toughness that it would normally have if it were crewed."
    );
    // Mech Hangar: "{3}, {T}: Target Vehicle becomes an artifact creature until end of
    // turn."
    let mut t = TestGame::new(2);
    let h = old(&mut t, P0, "Mech Hangar");
    let s = old(&mut t, P0, "Demolition Stomper");
    mana(&mut t, P0, ManaType::C, 3);
    t.answer_targets(P0, &[o(s)]);
    activate_text(&mut t, P0, h, "becomes an artifact creature").unwrap();
    t.resolve_all();
    assert!(creature_now(&t, s));
    assert_eq!(t.pt(s), (10, 7));
    // Mobilizer Mech: "Whenever this Vehicle becomes crewed, up to one other target
    // Vehicle you control becomes an artifact creature until end of turn."
    let mut t = TestGame::new(2);
    let m = old(&mut t, P0, "Mobilizer Mech");
    let s = old(&mut t, P0, "Demolition Stomper");
    let g = old(&mut t, P0, "Hill Giant");
    assert!(crew(&mut t, P0, m, &[g]));
    // (The trigger's target is chosen as it's put on the stack, after the crew ability
    // resolves.)
    t.answer_targets(P0, &[o(s)]);
    t.resolve_all();
    assert!(creature_now(&t, m));
    assert!(creature_now(&t, s));
    assert_eq!(t.pt(s), (10, 7));
}

#[test]
fn bespoke_battlewagon_tap_abilities_and_summoning_sickness() {
    cr!("302.6", "301.7");
    ruling!(
        "Bespoke Battlewagon",
        "As long as Bespoke Battlewagon isn't a creature, its {T} abilities can be activated even if you haven't controlled it continuously since the beginning of your most recent turn."
    );
    supported("Bespoke Battlewagon");
    let mut t = TestGame::new(2);
    let w = t.battlefield_sick(P0, "Bespoke Battlewagon");
    t.g.add_counters(pl(P0), counters::ENERGY, 4, None);
    assert!(can_activate_text(&mut t, P0, w, "You get {E}{E}"));
    activate_text(&mut t, P0, w, "Pay {E}{E}{E}{E}").unwrap();
    t.resolve_all();
    assert!(creature_now(&t, w));
    assert!(!can_activate_text(&mut t, P0, w, "You get {E}{E}"));
}

#[test]
fn demolition_stomper_stays_blocked_when_the_blockers_power_drops() {
    cr!("509.1h", "506.4");
    ruling!(
        "Demolition Stomper",
        "Once a creature with power 3 or greater has blocked Demolition Stomper, changing the power of the blocking creature won't cause Demolition Stomper to become unblocked."
    );
    let mut t = TestGame::new(2);
    let s = old(&mut t, P0, "Demolition Stomper");
    let a = old(&mut t, P0, "Hill Giant");
    let b = old(&mut t, P0, "Hill Giant");
    let foe = old(&mut t, P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(&mut t, s, &[a, b]);
    attack_with(&mut t, &[(s, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(foe, s)]));
    t.advance_to(P0, Step::DeclareBlockers);
    // Disfigure: -2/-2 on the blocker: power 1.
    cast_named(&mut t, P1, "Disfigure", &[o(foe)]);
    t.resolve_all();
    assert_eq!(t.pt(foe), (1, 1));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(foe));
}

#[test]
fn ghost_ark_grants_unearth_only_to_cards_there_as_it_resolves() {
    cr!("611.2c", "702.84a");
    ruling!(
        "Ghost Ark",
        "Ghost Ark's triggered ability gives unearth {3} to each artifact that is in your graveyard as it resolves."
    );
    supported("Ghost Ark");
    let mut t = TestGame::new(2);
    let ark = old(&mut t, P0, "Ghost Ark");
    let before = t.graveyard(P0, "Memnite");
    let bears = old(&mut t, P0, "Grizzly Bears");
    crewed(&mut t, ark, &[bears]);
    let after = t.graveyard(P0, "Ornithopter");
    mana(&mut t, P0, ManaType::C, 3);
    assert!(can_activate(&mut t, P0, before));
    assert!(!can_activate(&mut t, P0, after));
}

#[test]
fn boommobile_mana_on_one_ability_or_several() {
    cr!("106.6");
    ruling!(
        "Boommobile",
        "You may spend the four mana added by the first ability on the same ability or on multiple abilities."
    );
    supported("Boommobile");
    // One ability: its exhaust ability with X = 1 ({1}{2}{R}).
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    let b = enter(&mut t, P0, "Boommobile");
    t.resolve_all();
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::R), 4);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_targets(P0, &[pl(P1)]);
    activate_text(&mut t, P0, b, "Exhaust").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    // Several abilities: two Mishra's Foundry activations ({2} each).
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    enter(&mut t, P0, "Boommobile");
    t.resolve_all();
    let f1 = old(&mut t, P0, "Mishra's Foundry");
    let f2 = old(&mut t, P0, "Mishra's Foundry");
    for f in [f1, f2] {
        activate_text(&mut t, P0, f, "becomes a 2/2").unwrap();
        t.resolve_all();
        assert!(creature_now(&t, f));
    }
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    // Not spendable on a spell.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    enter(&mut t, P0, "Boommobile");
    t.resolve_all();
    let c = t.hand(P0, "Shock");
    assert!(t.cast(P0, c).target(pl(P1)).try_go().is_err());
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// The Lunar Whale
// ---------------------------------------------------------------------------

#[test]
fn the_lunar_whale_lets_you_look_at_the_top_card_any_time() {
    cr!("401.5", "116.1");
    ruling!(
        "The Lunar Whale",
        "The Lunar Whale lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority."
    );
    supported("The Lunar Whale");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    let top = t.library_top(P0, "Shock");
    t.battlefield(P0, "The Lunar Whale");
    t.g.recompute();
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    let sees = |t: &TestGame, p: PlayerId, id| mtg_engine::zones::can_see_in_library(&t.g, p, id);
    assert!(sees(&t, P0, top));
    assert!(!sees(&t, P1, top));
    // While the opponent has priority, after a draw, the new top card.
    t.set_step(P1, Step::PrecombatMain);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    let next = t.g.library_top(P0).unwrap();
    assert_eq!(t.g.obj(next).chars.name, "Grizzly Bears");
    assert!(sees(&t, P0, next));
    // The opponent watches what P0 knows: P0 was asked nothing to look.
    let _ = watch::<()>;
}

/// P0 crews The Lunar Whale with a Hill Giant and attacks with it, then goes to the
/// postcombat main phase.
fn whale_attacked(t: &mut TestGame) {
    let w = old(t, P0, "The Lunar Whale");
    let g = old(t, P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    crewed(t, w, &[g]);
    attack_with(t, &[(w, pl(P1))]);
    block_and_finish(t, P1, &[]);
}

#[test]
fn the_lunar_whale_top_card_follows_normal_timing_and_costs() {
    cr!("601.3", "305.2", "702.33a");
    ruling!(
        "The Lunar Whale",
        "You must follow all normal timing rules for spells cast and lands played using the third ability and must pay all costs for spells cast this way."
    );
    // A sorcery on top: not in combat, but in the main phase.
    let mut t = TestGame::new(2);
    let lava = t.library_top(P0, "Lava Axe");
    whale_attacked(&mut t);
    mana_all(&mut t, P0, 5);
    assert!(!can_cast(&mut t, P0, lava, CastMethod::Normal));
    t.advance_to(P0, Step::PostcombatMain);
    mana_all(&mut t, P0, 5);
    assert!(can_cast(&mut t, P0, lava, CastMethod::Normal));
    // It must be paid for.
    t.g.players[0].mana_pool.empty();
    assert!(!can_cast(&mut t, P0, lava, CastMethod::Normal));
    // A land on top: only with a land play left.
    let mut t = TestGame::new(2);
    let land = t.library_top(P0, "Forest");
    whale_attacked(&mut t);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, land));
    let f = t.hand(P0, "Plains");
    t.play_land(P0, f).unwrap();
    assert!(!can_play_land(&mut t, P0, land));
    // An optional additional cost (kicker) may be paid: Protect the Negotiators'
    // kicker isn't needed here — Burst Lightning: "Kicker {4}. Burst Lightning deals 2
    // damage to any target. If this spell was kicked, it deals 4 damage instead."
    let mut t = TestGame::new(2);
    let burst = t.library_top(P0, "Burst Lightning");
    whale_attacked(&mut t);
    t.advance_to(P0, Step::PostcombatMain);
    mana_all(&mut t, P0, 1);
    mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, burst).kicked(true).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
}

#[test]
fn crucible_of_worlds_doesnt_allow_cycling_from_the_graveyard() {
    cr!("702.29a", "305.1");
    ruling!(
        "Crucible of Worlds",
        "Crucible of Worlds doesn't allow you to activate abilities (such as cycling) of land cards in your graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crucible of Worlds");
    let s = t.graveyard(P0, "Secluded Steppe");
    mana_all(&mut t, P0, 2);
    assert!(!can_activate(&mut t, P0, s));
    assert!(can_play_land(&mut t, P0, s));
}
