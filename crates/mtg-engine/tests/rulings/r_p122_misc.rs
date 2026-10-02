//! Rulings batch P122 — assorted rulings of cards that stop attacks or blocks: Peace
//! Talks, Orim's Chant, Arboria (the world rule), Form of the Dragon, Blood Aspirant,
//! Clan Guildmage, Endbringer, Fifty Feet of Rope, Renegade Wheelsmith, Frenzied Goblin,
//! Grotag Thrasher, Burning-Tree Bloodscale, Auriok Siege Sled, Smelt-Ward Minotaur and
//! Markov Warlord.

use crate::r_p122_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::can_play_land;
use crate::r_s04_common::{can_cycle, stack_items};
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s21_common::{castable, legal_blocks};
use crate::r_s24_common::enter_together;
use crate::r_s27_common::can_activate_containing;
use crate::r_s28_common::cast_card;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn can_block(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.can_block_at_all(id)
}

// ---------------------------------------------------------------------------------
// Peace Talks and Orim's Chant.
// ---------------------------------------------------------------------------------

#[test]
fn peace_talks_after_your_attack_and_combat_still_happens() {
    cr!("611.2a", "508.1c", "506.1");
    ruling!("Peace Talks", "Can be cast after your attack.");
    ruling!(
        "Peace Talks",
        "Doesn’t skip the Combat Phase, just renders creatures unable to attack."
    );
    supported("Peace Talks");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::PostcombatMain);
    cast_card(&mut t, P0, "Peace Talks");
    t.resolve_all();
    // Next turn: the combat phase begins, but creatures can't attack.
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    // The turn after that, they can again.
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
}

#[test]
fn orims_chant_and_spells_cast_around_it() {
    cr!("101.2", "608.2c");
    ruling!(
        "Orim's Chant",
        "Orim's Chant won't affect spells that your opponents cast before you cast Orim's Chant, including any spells that are still on the stack. Orim's Chant also won't stop your opponents from casting spells after you cast Orim's Chant but before Orim's Chant resolves."
    );
    supported("Orim's Chant");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 3);
    // P1 casts Giant Growth; P0 responds with Orim's Chant; P1 responds with another.
    t.answer_targets(P1, &[obj(bears)]);
    let g1 = t.hand(P1, "Giant Growth");
    t.cast(P1, g1).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_card(&mut t, P0, "Orim's Chant");
    let g2 = t.hand(P1, "Giant Growth");
    assert!(castable(&mut t, P1, g2));
    t.answer_targets(P1, &[obj(bears)]);
    t.cast(P1, g2).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 8), "both Giant Growths resolved");
    let g3 = t.hand(P1, "Giant Growth");
    assert!(!castable(&mut t, P1, g3));
}

#[test]
fn orims_chant_doesnt_stop_abilities_or_land_plays() {
    cr!("101.2", "305.1");
    ruling!(
        "Orim's Chant",
        "The target opponent can still activate abilities, including abilities of cards in their hands (like cycling). Their triggered abilities work as normal, they can still play lands, and so on."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let meadow = t.hand(P1, "Drifting Meadow");
    let forest = t.hand(P1, "Forest");
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Plains", 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_card(&mut t, P0, "Orim's Chant");
    t.resolve_all();
    assert!(!castable(&mut t, P1, bears));
    assert!(can_cycle(&mut t, P1, meadow));
    assert!(can_activate_containing(&mut t, P1, sorcerer, "damage"));
    assert!(can_play_land(&mut t, P1, forest));
}

// ---------------------------------------------------------------------------------
// Arboria (the world rule) and Form of the Dragon.
// ---------------------------------------------------------------------------------

#[test]
fn arboria_world_rule() {
    cr!("704.5k");
    ruling!(
        "Arboria",
        "If two or more permanents have the world supertype, all of them are put into their owners’ graveyards except the one that has had the world supertype for the shortest amount of time. If there’s a tie for the shortest amount of time, all world permanents are put into the graveyard."
    );
    supported("Arboria");
    let mut t = TestGame::new(2);
    let old = t.enter(P0, "Arboria");
    t.settle();
    let new = t.enter(P1, "Arboria");
    t.settle();
    assert!(!t.on_battlefield(old));
    assert!(t.on_battlefield(new));
    // A tie: both are put into their owners' graveyards.
    let mut t = TestGame::new(2);
    let both = enter_together(&mut t, &[(P0, "Arboria"), (P1, "Arboria")]);
    assert!(both.iter().all(|a| !t.on_battlefield(*a)));
    assert!(t.in_graveyard(P0, "Arboria") && t.in_graveyard(P1, "Arboria"));
}

#[test]
fn form_of_the_dragon_sets_life_at_every_end_step() {
    cr!("119.5", "513.1");
    ruling!(
        "Form of the Dragon",
        "If you life total was above 5 at end of turn, then you lose life to make your total 5. If it was less than 5, you gain life to bring it to 5."
    );
    ruling!(
        "Form of the Dragon",
        "It sets your life total at the beginning of the end step of every player’s turn, not just your own."
    );
    supported("Form of the Dragon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Form of the Dragon");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 5);
    t.g.players[0].life = 2;
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 5, "on the opponent's turn too");
}

// ---------------------------------------------------------------------------------
// Blood Aspirant.
// ---------------------------------------------------------------------------------

#[test]
fn blood_aspirant_sacrificed_to_its_own_ability() {
    cr!("602.2b", "113.7a");
    ruling!(
        "Blood Aspirant",
        "Blood Aspirant can be sacrificed to pay for the cost of its own last ability."
    );
    supported("Blood Aspirant");
    let mut t = TestGame::new(2);
    let aspirant = t.battlefield(P0, "Blood Aspirant");
    let giant = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose(P0, &[obj(aspirant)]);
    t.activate(P0, aspirant, 0, &[obj(giant)]).unwrap();
    assert!(t.in_graveyard(P0, "Blood Aspirant"));
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 1);
    assert!(!can_block(&mut t, giant));
}

#[test]
fn blood_aspirant_trigger_before_the_ability() {
    cr!("603.3", "405.5");
    ruling!(
        "Blood Aspirant",
        "If you sacrifice a permanent as part of casting a spell or activating an ability, Blood Aspirant's first ability will resolve before that spell or ability."
    );
    ruling!(
        "Blood Aspirant",
        "you need some other way of sacrificing permanents, such as its second ability."
    );
    let mut t = TestGame::new(2);
    let aspirant = t.battlefield(P0, "Blood Aspirant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, aspirant, 0, &[obj(giant)]).unwrap();
    t.settle();
    let items = stack_items(&t);
    assert_eq!(items.len(), 2);
    assert!(items[1].contains("Whenever you sacrifice"), "{items:?}");
    t.resolve();
    assert_eq!(t.counters(aspirant, "+1/+1"), 1);
    assert_eq!(t.obj_now(giant).damage, 0, "the ability hasn't resolved yet");
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 1);
}

// ---------------------------------------------------------------------------------
// Clan Guildmage, Endbringer, Fifty Feet of Rope, Renegade Wheelsmith.
// ---------------------------------------------------------------------------------

#[test]
fn clan_guildmage_doesnt_untap_the_land() {
    cr!("110.5c");
    ruling!(
        "Clan Guildmage",
        "Clan Guildmage’s second ability doesn’t untap the land that becomes a creature."
    );
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Clan Guildmage");
    let mountain = t.battlefield(P0, "Mountain");
    t.g.tap(mountain);
    mana(&mut t, P0, ManaType::G, 3);
    t.activate(P0, mage, 1, &[obj(mountain)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(mountain), (4, 4));
    assert!(t.obj_now(mountain).tapped);
}

#[test]
fn endbringer_untaps_in_other_players_untap_steps() {
    cr!("502.3");
    ruling!(
        "Endbringer",
        "Endbringer untaps at the same time as the active player's permanents."
    );
    supported("Endbringer");
    let mut t = TestGame::new(2);
    let endbringer = t.battlefield(P0, "Endbringer");
    t.g.tap(endbringer);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(endbringer).tapped);
}

#[test]
fn endbringer_tied_up_still_untaps_on_another_players_turn() {
    cr!("502.3", "611.2a");
    ruling!(
        "Endbringer",
        "If an effect states that Endbringer doesn't untap during your untap step, that effect won't apply during another player's untap step."
    );
    ruling!(
        "Fifty Feet of Rope",
        "The Tie Up ability doesn't tap the creature, it just prevents it from untapping if it is already tapped or becomes tapped through some other means."
    );
    supported("Fifty Feet of Rope");
    let mut t = TestGame::new(2);
    let rope = t.battlefield(P0, "Fifty Feet of Rope");
    let endbringer = t.battlefield(P0, "Endbringer");
    // Tie Up doesn't tap the creature.
    mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, rope, 1, &[obj(endbringer)]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(endbringer).tapped);
    // Tapped, it untaps during P1's untap step: the effect is about its controller's.
    t.g.tap(endbringer);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(endbringer).tapped);
    // Tie Up on a tapped creature of P1's: it doesn't untap during P1's untap step.
    let mut t = TestGame::new(2);
    let rope = t.battlefield(P0, "Fifty Feet of Rope");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, rope, 1, &[obj(bears)]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn renegade_wheelsmith_must_become_tapped() {
    cr!("603.2e", "701.26a");
    ruling!(
        "Renegade Wheelsmith",
        "For the ability to trigger, Renegade Wheelsmith has to actually change from untapped to tapped. If an effect attempts to tap it, but it was already tapped at the time, this ability won’t trigger."
    );
    supported("Renegade Wheelsmith");
    let mut t = TestGame::new(2);
    let wheelsmith = t.battlefield(P0, "Renegade Wheelsmith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.g.tap(wheelsmith);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!can_block(&mut t, bears));
    // Already tapped: no trigger.
    t.g.tap(wheelsmith);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

// ---------------------------------------------------------------------------------
// Attack triggers and "can't block this creature".
// ---------------------------------------------------------------------------------

#[test]
fn frenzied_goblin_has_only_one_target() {
    cr!("115.1d", "603.5");
    ruling!(
        "Frenzied Goblin",
        "Frenzied Goblin's ability has only one target. You can't pay {R} multiple times to stop multiple creatures from blocking."
    );
    supported("Frenzied Goblin");
    use mtg_engine::decision::Decision;
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Frenzied Goblin");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Savannah Lions");
    t.answer_targets(P0, &[obj(a)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    to_combat(&mut t, P0);
    let from = t.asked().len();
    attack_with(&mut t, &[(goblin, Entity::Player(P1))]);
    mana(&mut t, P0, ManaType::R, 2);
    t.resolve_all();
    let asked = t.asked()[from..].to_vec();
    let target_choices: Vec<u32> = asked
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(target_choices, vec![1], "one target");
    let pays = asked
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(pays, 1, "{{R}} can be paid only once");
    assert!(!can_block(&mut t, a));
    assert!(can_block(&mut t, b));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1, "{{R}} paid once");
}

#[test]
fn grotag_thrasher_can_target_itself() {
    cr!("115.1d", "603.3d");
    ruling!(
        "Grotag Thrasher",
        "Grotag Thrasher’s ability can target any creature, not just one the defending player controls. For example, if you want that player’s creatures to be able to block this turn, you can target Grotag Thrasher with its own ability."
    );
    supported("Grotag Thrasher");
    let mut t = TestGame::new(2);
    let thrasher = t.battlefield(P0, "Grotag Thrasher");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(thrasher)]);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(thrasher, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!can_block(&mut t, thrasher));
    assert!(legal_blocks(&mut t, P1, &[(bears, thrasher)]));
}

/// `attacker_name` attacks; both of its abilities (`must` = "blocks this creature if
/// able", `cant` = "can't block this creature") target P1's `blocker_name`: it can't
/// block, and not blocking is legal.
fn cant_beats_must(attacker_name: &str, blocker_name: &str, must: (usize, ManaType), cant: (usize, ManaType)) {
    supported(attacker_name);
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, attacker_name);
    let blocker = t.battlefield(P1, blocker_name);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    mana(&mut t, P0, must.1, 3);
    t.activate(P0, attacker, must.0, &[obj(blocker)]).unwrap();
    t.resolve_all();
    // The requirement alone: not blocking is illegal.
    assert!(!legal_blocks(&mut t, P1, &[]), "{attacker_name}");
    mana(&mut t, P0, cant.1, 3);
    t.activate(P0, attacker, cant.0, &[obj(blocker)]).unwrap();
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(blocker, attacker)]), "{attacker_name}");
    assert!(legal_blocks(&mut t, P1, &[]), "{attacker_name}");
}

#[test]
fn burning_tree_bloodscale_both_abilities_on_one_creature() {
    cr!("509.1c", "101.2");
    ruling!(
        "Burning-Tree Bloodscale",
        "If both abilities are used on the same creature, it can't block Burning-Tree Bloodscale."
    );
    cant_beats_must(
        "Burning-Tree Bloodscale",
        "Grizzly Bears",
        (1, ManaType::G),
        (0, ManaType::R),
    );
}

#[test]
fn auriok_siege_sled_both_abilities_on_one_creature() {
    cr!("509.1c", "101.2");
    ruling!(
        "Auriok Siege Sled",
        "If you activate both of Auriok Siege Sled’s abilities on the same creature, that creature can’t block Auriok Siege Sled this turn."
    );
    cant_beats_must(
        "Auriok Siege Sled",
        "Ornithopter",
        (0, ManaType::C),
        (1, ManaType::C),
    );
}

#[test]
fn smelt_ward_minotaur_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Smelt-Ward Minotaur",
        "Smelt-Ward Minotaur’s triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Smelt-Ward Minotaur");
    let mine = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(bears)]);
    let growth = cast_card(&mut t, P0, "Giant Growth");
    t.settle();
    let items = stack_items(&t);
    assert_eq!(items[0], "Giant Growth");
    assert!(items[1].starts_with("ability"), "{items:?}");
    // P1 counters Giant Growth.
    t.answer_targets(P1, &[obj(growth)]);
    cast_card(&mut t, P1, "Cancel");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Giant Growth"));
    assert_eq!(t.pt(mine), (3, 3));
    assert!(!can_block(&mut t, bears));
}

#[test]
fn markov_warlord_targets_cant_block_any_creature() {
    cr!("509.1b");
    ruling!(
        "Markov Warlord",
        "The creatures can't block any creatures that turn, not just Markov Warlord."
    );
    supported("Markov Warlord");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Markov Warlord");
    t.g.flush_events();
    t.resolve_all();
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
}
