//! Rulings residue `rulings-residue-hate-attack-block` (P082): cards that punish blockers
//! or force blocks — The Wretched, Kjeldoran Frostbeast, Neyith of the Dire Hunt, Thicket
//! Basilisk, Basalt Golem, Slashing Tiger, Sorrow's Path, Lesser Werewolf, Dauthi Jackal,
//! Tetsuo Umezawa — and the batch's strays (Glen Elendra's Answer, Regicide).

use crate::r_p079_common::cast_card;
use crate::r_residue_hate_attack_block_common::*;
use crate::r_s01_common::supported;
use mtg_engine::card::card;
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::decision::Answer;
use mtg_engine::draft::{Draft, DraftInfo, DraftStyle};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// The Wretched
// ---------------------------------------------------------------------------

/// P0 attacks with The Wretched and P1 blocks it with `blockers`; stops in the declare
/// blockers step.
fn wretched_blocked_by(t: &mut TestGame, blockers: &[&str]) -> (ObjectId, Vec<ObjectId>) {
    supported("The Wretched");
    let w = t.battlefield(P0, "The Wretched");
    let bs: Vec<ObjectId> = blockers.iter().map(|b| t.battlefield(P1, b)).collect();
    let decl: Vec<(ObjectId, ObjectId)> = bs.iter().map(|b| (*b, w)).collect();
    blocks(t, &[w], &decl);
    (w, bs)
}

/// P0 casts Fog (no combat damage this turn).
fn fog(t: &mut TestGame) {
    cast_card(t, P0, "Fog", &[]).unwrap();
    t.resolve_all();
}

#[test]
fn the_wretched_takes_creatures_put_onto_the_battlefield_blocking_it() {
    cr!("509.4", "611.2b");
    ruling!(
        "The Wretched",
        "The ability grants you control of all creatures that are blocking it as the ability resolves. This will include any creatures that were put onto the battlefield blocking it."
    );
    let mut t = TestGame::new(2);
    let (w, bs) = wretched_blocked_by(&mut t, &["Wall of Wood"]);
    cast_card(&mut t, P1, "Flash Foliage", &[obj(w)]).unwrap();
    t.resolve_all();
    let saproling = crate::r_s01_common::tokens(&t, P1);
    assert_eq!(saproling.len(), 1);
    assert_eq!(blocking(&t, saproling[0]), vec![w]);
    fog(&mut t);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(controller(&t, bs[0]), P0);
    assert_eq!(controller(&t, saproling[0]), P0);
}

#[test]
fn the_wretched_must_survive_combat_damage() {
    cr!("511.1", "603.2");
    ruling!(
        "The Wretched",
        "The Wretched’s ability triggers only if it’s still on the battlefield when the end of combat step begins (after the combat damage step). For example, if it’s blocked by a 7/7 creature and is destroyed, its ability won’t trigger at all."
    );
    let mut t = TestGame::new(2);
    let (w, bs) = wretched_blocked_by(&mut t, &["Craw Wurm"]);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert!(!t.on_battlefield(w));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(controller(&t, bs[0]), P1);
}

#[test]
fn the_wretched_regenerated_is_out_of_combat_and_takes_nothing() {
    cr!("506.4", "701.19c");
    ruling!(
        "The Wretched",
        "If The Wretched itself regenerated during combat, then it will have been removed from combat. Since it is no longer in combat, there cannot be any creatures blocking it, which means you won’t be able to gain control of any creatures."
    );
    let mut t = TestGame::new(2);
    let (w, bs) = wretched_blocked_by(&mut t, &["Craw Wurm"]);
    cast_card(&mut t, P0, "Regenerate", &[obj(w)]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(w), "{}", t.dump_log());
    assert_eq!(controller(&t, bs[0]), P1);
}

#[test]
fn the_wretched_lost_before_its_ability_resolves_takes_nothing() {
    cr!("611.2b");
    ruling!(
        "The Wretched",
        "If you lose control of The Wretched before its ability resolves, you won’t gain control of the creatures blocking it at all."
    );
    let mut t = TestGame::new(2);
    let (w, bs) = wretched_blocked_by(&mut t, &["Wall of Wood"]);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    give_control(&mut t, w, P1);
    t.resolve_all();
    assert_eq!(controller(&t, bs[0]), P1);
}

#[test]
fn the_wretched_control_ends_when_it_leaves() {
    cr!("611.2b");
    ruling!(
        "The Wretched",
        "If The Wretched leaves the battlefield, you no longer control it, so the duration of its control-change effect ends."
    );
    let mut t = TestGame::new(2);
    let (w, bs) = wretched_blocked_by(&mut t, &["Wall of Wood"]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(controller(&t, bs[0]), P0);
    crate::r_s02_common::destroy(&mut t, w);
    assert_eq!(controller(&t, bs[0]), P1);
}

#[test]
fn the_wretched_keeps_permanents_that_stop_being_creatures() {
    cr!("611.2b", "611.2c");
    ruling!(
        "The Wretched",
        "Once the ability resolves, it doesn’t care whether the permanents you gained control of remain creatures, only that they remain on the battlefield."
    );
    supported("Treetop Village");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "The Wretched");
    let village = t.battlefield(P1, "Treetop Village");
    t.lands(P1, "Forest", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s06_common::activate_containing(&mut t, P1, village, "becomes").expect("animate");
    t.resolve_all();
    t.g.obj_mut(village).tapped = false;
    blocks(&mut t, &[w], &[(village, w)]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(controller(&t, village), P0, "{}", t.dump_log());
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(village).is_creature());
    assert_eq!(controller(&t, village), P0);
}

// ---------------------------------------------------------------------------
// Kjeldoran Frostbeast
// ---------------------------------------------------------------------------

#[test]
fn kjeldoran_frostbeast_needs_to_survive_and_be_in_combat() {
    cr!("511.1", "603.2", "506.4");
    ruling!(
        "Kjeldoran Frostbeast",
        "Kjeldoran Frostbeast’s ability triggers only if it’s still on the battlefield when the end of combat step begins (after the combat damage step). For example, if it blocks a 7/7 creature and is destroyed, its ability won’t trigger at all. The 7/7 will remain on the battlefield."
    );
    ruling!(
        "Kjeldoran Frostbeast",
        "Its ability will destroy all creatures it’s currently blocking or blocked by, not necessarily the creatures it had blocked or become blocked by during that combat phase. For example, if Kjeldoran Frostbeast is dealt lethal combat damage and regenerates, it’s removed from combat. Its ability will trigger at end of combat, but since it’s no longer blocking or blocked by any creatures, the ability won’t do anything."
    );
    supported("Kjeldoran Frostbeast");
    // Blocking a Craw Wurm, it's destroyed: no trigger.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let beast = t.battlefield(P1, "Kjeldoran Frostbeast");
    blocks(&mut t, &[wurm], &[(beast, wurm)]);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert!(!t.on_battlefield(beast));
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(wurm));
    // Regenerated: out of combat, its ability does nothing.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let beast = t.battlefield(P1, "Kjeldoran Frostbeast");
    blocks(&mut t, &[wurm], &[(beast, wurm)]);
    cast_card(&mut t, P1, "Regenerate", &[obj(beast)]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(
        t.on_battlefield(beast) && t.on_battlefield(wurm),
        "{}",
        t.dump_log()
    );
    // Still blocking at end of combat: the attacker is destroyed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let beast = t.battlefield(P1, "Kjeldoran Frostbeast");
    blocks(&mut t, &[bears], &[(beast, bears)]);
    fog(&mut t);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------
// Neyith of the Dire Hunt: "must be blocked this combat if able"
// ---------------------------------------------------------------------------

/// P0 controls Neyith; at the beginning of combat P0 pays {2}{R/G} and targets
/// `target`; then P0 attacks P1 with `attackers` and the game stops in the declare
/// attackers step.
fn neyith_targets(t: &mut TestGame, target: ObjectId, attackers: &[ObjectId]) {
    supported("Neyith of the Dire Hunt");
    t.battlefield(P0, "Neyith of the Dire Hunt");
    t.lands(P0, "Mountain", 3);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(target)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let at: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    attacks(t, &at);
}

fn legal_blocks(t: &mut TestGame, decl: &[(ObjectId, ObjectId)]) -> bool {
    t.g.recompute();
    let opts = block_options(&t.g, &[P1]);
    block_declaration_legal(&t.g, &opts, decl)
}

#[test]
fn neyith_requires_one_blocker_of_the_defenders_choice() {
    cr!("509.1c", "509.1a");
    ruling!(
        "Neyith of the Dire Hunt",
        "Only one creature is required to block the target creature. Other creatures may also block it and are free to block other creatures or not block at all."
    );
    ruling!(
        "Neyith of the Dire Hunt",
        "The defending player, not you, chooses which creature blocks the target creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    let elves = t.battlefield(P1, "Llanowar Elves");
    neyith_targets(&mut t, bears, &[bears, giant]);
    assert_eq!(t.pt(bears), (4, 2), "{}", t.dump_log());
    assert!(!legal_blocks(&mut t, &[]));
    assert!(!legal_blocks(&mut t, &[(wall, giant)]));
    assert!(legal_blocks(&mut t, &[(wall, bears)]));
    assert!(legal_blocks(&mut t, &[(elves, bears)]));
    assert!(legal_blocks(&mut t, &[(wall, bears), (elves, bears)]));
    assert!(legal_blocks(&mut t, &[(wall, bears), (elves, giant)]));
}

#[test]
fn neyith_doesnt_force_a_block_that_cant_be_made_or_costs_something() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Neyith of the Dire Hunt",
        "If each creature the defending player controls can’t block for any reason (such as being tapped), then the target creature isn’t blocked. If there’s a cost associated with blocking the target creature, the defending player isn’t forced to pay that cost, so it doesn’t have to be blocked in that case either."
    );
    // Tapped.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    tap(&mut t, wall);
    neyith_targets(&mut t, bears, &[bears]);
    assert!(legal_blocks(&mut t, &[]));
    // Blocking costs {1} (War Cadence).
    let mut t = TestGame::new(2);
    let cadence = t.battlefield(P0, "War Cadence");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Wall of Wood");
    t.lands(P0, "Mountain", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, cadence, 0, &[]).unwrap();
    t.resolve_all();
    neyith_targets(&mut t, bears, &[bears]);
    assert!(legal_blocks(&mut t, &[]), "{}", t.dump_log());
}

#[test]
fn neyith_target_with_menace_needs_two_blockers() {
    cr!("509.1c", "702.111b");
    ruling!(
        "Neyith of the Dire Hunt",
        "If the target creature has menace, two creatures must block it if able."
    );
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Boggart Brute");
    let wall = t.battlefield(P1, "Wall of Wood");
    let elves = t.battlefield(P1, "Llanowar Elves");
    neyith_targets(&mut t, brute, &[brute]);
    assert!(!legal_blocks(&mut t, &[]));
    assert!(!legal_blocks(&mut t, &[(wall, brute)]));
    assert!(legal_blocks(&mut t, &[(wall, brute), (elves, brute)]));
}

// ---------------------------------------------------------------------------
// Other block punishers
// ---------------------------------------------------------------------------

#[test]
fn thicket_basilisk_destroys_a_protected_blocker() {
    cr!("702.16b", "603.7");
    ruling!(
        "Thicket Basilisk",
        "Protection from Green does not stop the Basilisk’s ability because the ability is not targeted."
    );
    // Mirran Crusader: protection from black and from green.
    let mut t = TestGame::new(2);
    let basilisk = t.battlefield(P0, "Thicket Basilisk");
    let crusader = t.battlefield(P1, "Mirran Crusader");
    blocks(&mut t, &[basilisk], &[(crusader, basilisk)]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(crusader), "{}", t.dump_log());
}

#[test]
fn basalt_golem_wall_only_if_the_blocker_was_sacrificed() {
    cr!("608.2c", "603.7");
    ruling!(
        "Basalt Golem",
        "The token creature is put onto the battlefield at the end of combat only if the blocking creature is sacrificed during the resolution of the triggered ability."
    );
    supported("Basalt Golem");
    // The Bears die in combat: nothing to sacrifice, no Wall. The Wall of Wood survives:
    // sacrificed, and P1 gets a Wall token.
    for (blocker, wall) in [("Grizzly Bears", false), ("Wall of Wood", true)] {
        let mut t = TestGame::new(2);
        let golem = t.battlefield(P0, "Basalt Golem");
        let b = t.battlefield(P1, blocker);
        blocks(&mut t, &[golem], &[(b, golem)]);
        t.resolve_all();
        t.advance_to(P0, Step::EndOfCombat);
        t.resolve_all();
        assert!(!t.on_battlefield(b), "{blocker}");
        assert_eq!(
            crate::r_s01_common::tokens(&t, P1).len(),
            wall as usize,
            "{blocker}: {}",
            t.dump_log()
        );
    }
}

#[test]
fn block_punisher_restatements() {
    cr!("509.3c", "602.2");
    ruling!(
        "Slashing Tiger",
        "When Slashing Tiger becomes blocked, the ability will trigger just once no matter how many creatures are blocking it."
    );
    ruling!(
        "Dauthi Jackal",
        "This creature does not have to be an attacker or blocker to use its ability."
    );
    // Slashing Tiger blocked by two creatures: one trigger.
    let mut t = TestGame::new(2);
    let tiger = t.battlefield(P0, "Slashing Tiger");
    let w1 = t.battlefield(P1, "Wall of Wood");
    let w2 = t.battlefield(P1, "Wall of Wood");
    blocks(&mut t, &[tiger], &[(w1, tiger), (w2, tiger)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(tiger), (5, 5));
    // Dauthi Jackal that isn't in combat destroys a blocking creature.
    let mut t = TestGame::new(2);
    let jackal = t.battlefield(P0, "Dauthi Jackal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.lands(P0, "Swamp", 2);
    blocks(&mut t, &[bears], &[(wall, bears)]);
    t.activate(P0, jackal, 0, &[obj(wall)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(wall) && !t.on_battlefield(jackal));
}

#[test]
fn sorrows_path_damage_trigger_resolves_before_the_exchange() {
    cr!("603.3", "405.5");
    ruling!(
        "Sorrow's Path",
        "When Sorrow’s Path’s first ability is activated, its second ability triggers and goes on the stack on top of the first ability. The second ability resolves first, and may cause some of the attacking creatures to be dealt lethal damage."
    );
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    let elves = t.battlefield(P1, "Llanowar Elves");
    blocks(&mut t, &[bears, giant], &[(wall, bears), (elves, giant)]);
    t.answer_targets(P0, &[obj(wall), obj(elves)]);
    t.activate(P0, path, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The trigger resolved first: 2 damage to P0 and each creature P0 controls.
    assert_eq!(t.life(P0), 18);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(blocking(&t, wall), vec![giant]);
    assert!(blocking(&t, elves).is_empty());
}

#[test]
fn lesser_werewolf_activates_at_any_power_and_does_nothing_at_zero() {
    cr!("602.2", "608.2c");
    ruling!(
        "Lesser Werewolf",
        "You may activate Lesser Werewolf’s ability no matter what its power is. However, if Lesser Werewolf’s power is 0 or less at the time the ability resolves, the ability won’t do anything."
    );
    supported("Lesser Werewolf");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wolf = t.battlefield(P1, "Lesser Werewolf");
    t.lands(P1, "Swamp", 4);
    blocks(&mut t, &[giant], &[(wolf, giant)]);
    for _ in 0..3 {
        t.activate(P1, wolf, 0, &[obj(giant)]).unwrap();
    }
    t.resolve_all();
    assert_eq!(t.pt(wolf), (0, 4));
    assert_eq!(t.pt(giant), (3, 1), "{}", t.dump_log());
    // At power 0 it can still be activated; it does nothing.
    t.activate(P1, wolf, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(wolf), (0, 4));
    assert_eq!(t.pt(giant), (3, 1));
}

#[test]
fn tetsuo_umezawa_can_be_targeted_by_an_auras_ability() {
    cr!("115.1", "113.7");
    ruling!(
        "Tetsuo Umezawa",
        "Tetsuo Umezawa can be targeted by abilities from Auras."
    );
    supported("Tetsuo Umezawa");
    supported("Crackling Club");
    let mut t = TestGame::new(2);
    let tetsuo = t.battlefield(P1, "Tetsuo Umezawa");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let club = crate::r_s06_common::attach_new(&mut t, P0, "Crackling Club", bears);
    // An Aura spell can't target it.
    let strength = t.hand(P0, "Holy Strength");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Holy Strength");
    let _ = t.cast(P0, strength).target(tetsuo).try_go();
    t.resolve_all();
    assert!(t.g.attachments_of(obj(t.g.current(tetsuo))).is_empty());
    // The Aura's ability can.
    t.activate(P0, club, 0, &[obj(tetsuo)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(tetsuo).damage, 1);
}

#[test]
fn glen_elendras_answer_targeted_by_a_counterspell_isnt_countered() {
    cr!("101.2", "701.6a");
    ruling!(
        "Glen Elendra's Answer",
        "A spell or ability that counters spells can still target Glen Elendra's Answer. When that spell or ability resolves, Glen Elendra's Answer won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    supported("Glen Elendra's Answer");
    let mut t = TestGame::new(2);
    cast_card(&mut t, P1, "Shock", &[Entity::Player(P0)]).unwrap();
    let answer = cast_card(&mut t, P0, "Glen Elendra's Answer", &[]).unwrap();
    let hand = t.hand_size(P1);
    cast_card(&mut t, P1, "Dismiss", &[obj(answer)]).unwrap();
    t.resolve();
    // Dismiss resolved: P1 drew a card; the Answer is still on the stack.
    assert_eq!(t.hand_size(P1), hand + 1);
    assert!(t.g.stack.contains(&answer));
    t.resolve_all();
    // The Answer countered Shock and made a Faerie for it.
    assert_eq!(t.life(P0), 20);
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

// ---------------------------------------------------------------------------
// Regicide (Conspiracy draft colors, CR 905.2b)
// ---------------------------------------------------------------------------

/// Three players, one round: P0's pack is `p0`, P1's and P2's are `p1`/`p2`; each player
/// drafts in pack order, with the colors preset for each Regicide `colors` (by player, in
/// the order they draft them). Returns the game.
fn drafted(p0: &[&str], p1: &[&str], p2: &[&str], colors: &[(PlayerId, [Color; 3])]) -> TestGame {
    let pack = |names: &[&str]| -> Vec<std::sync::Arc<mtg_engine::card::CardDef>> {
        names.iter().map(|n| card(n)).collect()
    };
    let mut d = Draft::new(
        vec![vec![pack(p0)], vec![pack(p1)], vec![pack(p2)]],
        DraftStyle::Conspiracy,
        1,
    );
    let mut colors = colors.to_vec();
    while !d.is_complete() {
        for p in [P0, P1, P2] {
            if d.pack(p).is_empty() {
                continue;
            }
            let regicide = d.pack(p)[0].name == "Regicide";
            if regicide {
                if let Some(i) = colors.iter().position(|(q, _)| *q == p) {
                    let (_, c) = colors.remove(i);
                    d.preset_colors(p, c);
                }
            }
            d.pick(p, &[0]).unwrap();
        }
    }
    let (_, info): (_, DraftInfo) = d.finish();
    let mut t = TestGame::new(3);
    t.g.start.draft = info;
    t.set_step(P0, Step::PrecombatMain);
    t
}

#[test]
fn regicide_colors_are_three_different_ones() {
    cr!("905.2b");
    ruling!(
        "Regicide",
        "The three chosen colors must be different from one another."
    );
    interp!("draft-three-colors-default");
    supported("Regicide");
    // A choice repeating a color isn't allowed: three different colors are chosen.
    let t = drafted(
        &["Regicide"],
        &["Grizzly Bears"],
        &["Grizzly Bears"],
        &[(P0, [Color::Red, Color::Red, Color::White])],
    );
    let mut c = t.g.start.draft.notes[0].colors.clone();
    assert_eq!(c.len(), 3);
    c.sort();
    c.dedup();
    assert_eq!(c.len(), 3);
}

#[test]
fn regicide_colors_of_every_regicide_drafted_count() {
    cr!("905.2b");
    ruling!(
        "Regicide",
        "If you draft another Regicide, any additional colors chosen will count for each Regicide you've drafted."
    );
    let mut t = drafted(
        // P0 drafts the first; the pack passed to P0 has the second.
        &["Regicide", "Grizzly Bears", "Grizzly Bears"],
        &["Grizzly Bears", "Regicide", "Grizzly Bears"],
        &["Grizzly Bears", "Regicide", "Grizzly Bears"],
        &[
            (P0, [Color::Red, Color::White, Color::Green]),
            (P0, [Color::Blue, Color::Red, Color::Green]),
        ],
    );
    let lions = t.battlefield(P1, "Savannah Lions");
    let looter = t.battlefield(P1, "Merfolk Looter");
    for target in [lions, looter] {
        cast_card(&mut t, P0, "Regicide", &[obj(target)]).unwrap();
        t.resolve_all();
        assert!(!t.on_battlefield(target));
    }
}

#[test]
fn regicide_without_having_drafted_one_has_no_legal_target() {
    cr!("905.2b", "601.2c");
    ruling!(
        "Regicide",
        "If a card would allow you to cast a Regicide but you didn't draft one, you won't be able to cast it because you won't be able to choose a legal target."
    );
    let mut t = drafted(
        &["Regicide"],
        &["Grizzly Bears"],
        &["Grizzly Bears"],
        &[(P0, [Color::Red, Color::White, Color::Green])],
    );
    let lions = t.battlefield(P0, "Savannah Lions");
    // P1 drafted no Regicide.
    t.set_step(P1, Step::PrecombatMain);
    assert!(cast_card(&mut t, P1, "Regicide", &[obj(lions)]).is_err());
    assert!(t.on_battlefield(lions));
}

#[test]
fn regicide_copy_uses_its_controllers_drafted_colors() {
    cr!("905.2b", "707.10c", "608.2b");
    ruling!(
        "Regicide",
        "If you copy or gain control of a Regicide that's already on the stack but you didn't draft one, you won't be able to choose a new target and Regicide won't resolve as it tries to resolve. The target creature won't be destroyed. On the other hand, if you did draft a Regicide, use the set of colors chosen as you drafted cards named Regicide when choosing a new target (if applicable) or determining if the resolving Regicide's target is legal."
    );
    // P0 drafted Regicide with red, white, green; P2 with blue, black, green; P1 none.
    let setup = || {
        let mut t = drafted(
            &["Regicide"],
            &["Grizzly Bears"],
            &["Regicide"],
            &[
                (P0, [Color::Red, Color::White, Color::Green]),
                (P2, [Color::Blue, Color::Black, Color::Green]),
            ],
        );
        let lions = t.battlefield(P1, "Savannah Lions");
        let looter = t.battlefield(P1, "Merfolk Looter");
        let regicide = cast_card(&mut t, P0, "Regicide", &[obj(lions)]).unwrap();
        (t, lions, looter, regicide)
    };
    // P1 copies it: the copy's target (white) isn't legal for P1, so the copy doesn't
    // resolve.
    let (mut t, lions, _, regicide) = setup();
    cast_card(&mut t, P1, "Twincast", &[obj(regicide)]).unwrap();
    t.answer_yes(P1, true);
    t.resolve();
    t.resolve();
    assert!(t.g.stack.contains(&regicide), "the copy is on top");
    assert!(t.on_battlefield(lions), "{}", t.dump_log());
    // P2 copies it and chooses a new target with P2's colors: the blue Looter.
    let (mut t, lions, looter, regicide) = setup();
    cast_card(&mut t, P2, "Twincast", &[obj(regicide)]).unwrap();
    t.answer_yes(P2, true);
    t.answer_targets(P2, &[obj(looter)]);
    t.resolve();
    t.resolve();
    assert!(!t.on_battlefield(looter), "{}", t.dump_log());
    assert!(t.on_battlefield(lions));
    // P2 copies it keeping the target: white isn't one of P2's colors.
    let (mut t, lions, _, regicide) = setup();
    cast_card(&mut t, P2, "Twincast", &[obj(regicide)]).unwrap();
    t.answer_yes(P2, false);
    t.resolve();
    t.resolve();
    assert!(t.g.stack.contains(&regicide));
    assert!(t.on_battlefield(lions));
}
