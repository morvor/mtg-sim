//! Rulings batch P117 — abilities that use their source's power as the ability resolves:
//! if the source has left the battlefield, its power as it last existed there is used
//! (CR 608.2h, 603.10a, 113.7a). Each test triggers the ability, then in response gives
//! the source +1/+1 counters and removes it, so the last known power differs from both
//! the printed power and the power of the card in its new zone.

use crate::r_p117_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- attack triggers ------------------------------------------------------------------

#[test]
fn ashroot_animist_x_is_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Ashroot Animist",
        "If Ashroot Animist leaves the battlefield before its last ability resolves, use its power as it last existed on the battlefield to determine the value of X."
    );
    supported("Ashroot Animist");
    let mut t = TestGame::new(2);
    let animist = t.battlefield(P0, "Ashroot Animist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(animist, Entity::Player(P1))]);
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, animist, 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2 + 6, 2 + 6));
}

#[test]
fn gruul_beastmaster_x_is_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Gruul Beastmaster",
        "If Gruul Beastmaster leaves the battlefield before its triggered ability resolves, use its power as it last existed on the battlefield to determine the value of X."
    );
    supported("Gruul Beastmaster");
    let mut t = TestGame::new(2);
    let gruul = t.battlefield(P0, "Gruul Beastmaster");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(gruul, Entity::Player(P1))]);
    grow_then_exile(&mut t, gruul, 3);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2 + 5, 2));
}

#[test]
fn syr_faren_x_is_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Syr Faren, the Hengehammer",
        "If Syr Faren leaves the battlefield before its triggered ability resolves, use its power as it last existed on the battlefield to determine the value of X."
    );
    supported("Syr Faren, the Hengehammer");
    let mut t = TestGame::new(2);
    let faren = t.battlefield(P0, "Syr Faren, the Hengehammer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(
        &mut t,
        &[(faren, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    grow_then_exile(&mut t, faren, 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2 + 4, 2 + 4));
}

#[test]
fn krenko_gets_no_counter_but_uses_its_last_known_power() {
    cr!("608.2h", "113.7a", "608.2b");
    ruling!(
        "Krenko, Tin Street Kingpin",
        "If Krenko leaves the battlefield after its ability has triggered but before it resolves, you don't put a +1/+1 counter on it, but you do use its power as it last existed before it left the battlefield to determine how many Goblin tokens to create."
    );
    supported("Krenko, Tin Street Kingpin");
    let mut t = TestGame::new(2);
    let krenko = t.battlefield(P0, "Krenko, Tin Street Kingpin");
    attack_with(&mut t, &[(krenko, Entity::Player(P1))]);
    grow_then_exile(&mut t, krenko, 2);
    t.resolve_all();
    // 1 + 2 counters; no further counter.
    assert_eq!(tokens_named(&t, P0, "Goblin"), 3);
    assert_eq!(plus1(&t, krenko), 0, "the exiled card has no counters");
}

#[test]
fn exuberant_wolfbear_uses_its_last_known_power_and_toughness() {
    cr!("608.2h", "113.7a", "613.4b");
    ruling!(
        "Exuberant Wolfbear",
        "If Exuberant Wolfbear leaves the battlefield while its triggered ability is on the stack, use its power and toughness as it last existed on the battlefield to determine the Human’s new base power and toughness."
    );
    supported("Exuberant Wolfbear");
    let mut t = TestGame::new(2);
    let wolfbear = t.battlefield(P0, "Exuberant Wolfbear");
    let human = t.battlefield(P0, "Elite Vanguard");
    t.answer_targets(P0, &[obj(human)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(wolfbear, Entity::Player(P1))]);
    grow_then_exile(&mut t, wolfbear, 2);
    t.resolve_all();
    assert_eq!(t.pt(human), (6, 6));
}

#[test]
fn tanuki_transplanter_adds_mana_equal_to_the_last_known_power() {
    cr!("608.2h", "113.7a", "106.4");
    ruling!(
        "Tanuki Transplanter",
        "If Tanuki Transplanter attacks, use its power to determine how much {G} to add. If equipped creature attacks, use its power. In either case, use the power as the triggered ability resolves."
    );
    supported("Tanuki Transplanter");
    // Tanuki Transplanter itself attacks; it leaves in response.
    let mut t = TestGame::new(2);
    let tanuki = t.battlefield(P0, "Tanuki Transplanter");
    attack_with(&mut t, &[(tanuki, Entity::Player(P1))]);
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, tanuki, 2);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 4);
    // The equipped creature attacks: its power as the ability resolves (after a pump).
    let mut t = TestGame::new(2);
    let tanuki = t.battlefield(P0, "Tanuki Transplanter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach_as_creature(tanuki, Entity::Object(bears)));
    t.g.recompute();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(stack(&t), 1);
    giant_growth(&mut t, bears);
    assert_eq!(stack(&t), 1);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 5);
}

#[test]
fn tanuki_transplanter_equipped_creature_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Tanuki Transplanter",
        "Also in either case, if the creature isn't on the battlefield at that time, use its power as it last existed on the battlefield to determine how much {G} to add."
    );
    let mut t = TestGame::new(2);
    let tanuki = t.battlefield(P0, "Tanuki Transplanter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach_as_creature(tanuki, Entity::Object(bears)));
    t.g.recompute();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    grow_then_exile(&mut t, bears, 2);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 4);
}

#[test]
fn shadowfax_compares_with_its_last_known_power() {
    cr!("608.2h", "113.7a", "508.4");
    ruling!(
        "Shadowfax, Lord of Horses",
        "If Shadowfax, Lord of Horses leaves the battlefield before its triggered ability resolves, compare the power of the creature card in your hand with Shadowfax's power as it last existed on the battlefield."
    );
    supported("Shadowfax, Lord of Horses");
    let mut t = TestGame::new(2);
    let shadowfax = t.battlefield(P0, "Shadowfax, Lord of Horses");
    let tusk = t.hand(P0, "Thragtusk");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(tusk)]);
    attack_with(&mut t, &[(shadowfax, Entity::Player(P1))]);
    grow_then_exile(&mut t, shadowfax, 2);
    t.resolve_all();
    // Thragtusk (power 5) has lesser power than a 6-power Shadowfax.
    let tusk = t.g.current(tusk);
    assert!(t.on_battlefield(tusk));
    assert!(t.g.combat.as_ref().unwrap().attacker(tusk).is_some());
}

#[test]
fn dreadhorde_arcanist_target_legality_uses_its_last_known_power() {
    cr!("608.2b", "113.7a", "608.2h");
    ruling!(
        "Dreadhorde Arcanist",
        "If Dreadhorde Arcanist leaves the battlefield before its triggered ability resolves, use its power as it last existed on the battlefield to check whether the card is still a legal target."
    );
    supported("Dreadhorde Arcanist");
    let mut t = TestGame::new(2);
    let arcanist = with_counters(&mut t, P0, "Dreadhorde Arcanist", 2);
    let div = t.graveyard(P0, "Divination");
    t.answer_targets(P0, &[obj(div)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(arcanist, Entity::Player(P1))]);
    assert_eq!(stack(&t), 1);
    exile(&mut t, arcanist);
    let hand = t.hand_size(P0);
    t.resolve_all();
    // Divination (mana value 3) is still legal for a 3-power Arcanist: cast and resolved.
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_exile("Divination"));
}

#[test]
fn ayesha_tanaka_uses_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Ayesha Tanaka, Armorer",
        "Ayesha's power is checked only once, as the triggered ability resolves. If it is no longer on the battlefield at that point, use its power as it last existed on the battlefield."
    );
    supported("Ayesha Tanaka, Armorer");
    let mut t = TestGame::new(2);
    let ayesha = t.battlefield(P0, "Ayesha Tanaka, Armorer");
    let lib = stack_library(
        &mut t,
        P0,
        &["Solemn Simulacrum", "Forest", "Forest", "Forest"],
    );
    t.answer_choose(P0, &[obj(lib[0])]);
    attack_with(&mut t, &[(ayesha, Entity::Player(P1))]);
    grow_then_exile(&mut t, ayesha, 2);
    t.resolve_all();
    // Solemn Simulacrum (mana value 4) ≤ a 4-power Ayesha.
    assert_eq!(t.named_on_battlefield("Solemn Simulacrum").len(), 1);
}

#[test]
fn ayesha_tanaka_compares_each_card_not_the_total() {
    cr!("608.2c");
    ruling!(
        "Ayesha Tanaka, Armorer",
        "Each of the artifact cards needs to have mana value less than or equal to Ayesha's Power. It's not the total mana value of all the artifact cards."
    );
    let mut t = TestGame::new(2);
    let ayesha = t.battlefield(P0, "Ayesha Tanaka, Armorer");
    let lib = stack_library(
        &mut t,
        P0,
        &["Mind Stone", "Mind Stone", "Forest", "Solemn Simulacrum"],
    );
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    let from = n_asked(&t);
    attack_with(&mut t, &[(ayesha, Entity::Player(P1))]);
    t.resolve_all();
    let offered: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&obj(lib[0])) && offered[0].contains(&obj(lib[1])));
    assert!(!offered[0].contains(&obj(lib[3])));
    // Two Mind Stones (2 each, 4 total) for a 2-power Ayesha; not Solemn (4).
    assert_eq!(t.named_on_battlefield("Mind Stone").len(), 2);
    assert!(t.named_on_battlefield("Solemn Simulacrum").is_empty());
}

#[test]
fn meglonoth_deals_damage_equal_to_its_last_known_power() {
    cr!("608.2h", "113.7a", "509.1");
    ruling!(
        "Meglonoth",
        "If Meglonoth leaves the battlefield after its ability triggers but before it resolves, the amount of damage it deals is equal to its power as it last existed on the battlefield."
    );
    supported("Meglonoth");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let meg = t.battlefield(P0, "Meglonoth");
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(meg, bears)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    t.settle();
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, meg, 2);
    t.resolve_all();
    assert_eq!(lost(&t, P1), 8);
}

#[test]
fn syr_faren_negative_power_gives_plus_zero() {
    cr!("107.1b", "608.2h");
    ruling!(
        "Syr Faren, the Hengehammer",
        "If Syr Faren's power is negative as its triggered ability resolves, X is considered to be 0."
    );
    let mut t = TestGame::new(2);
    let faren = t.battlefield(P0, "Syr Faren, the Hengehammer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(
        &mut t,
        &[(faren, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    minus_power(&mut t, faren, 4);
    assert_eq!(t.pt(faren).0, -2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn gruul_beastmaster_negative_power_gives_plus_zero() {
    cr!("107.1b", "608.2h");
    ruling!(
        "Gruul Beastmaster",
        "If Gruul Beastmaster’s power is negative as its triggered ability resolves, X is considered to be 0. In other words, that ability can’t cause the creature to lose power."
    );
    let mut t = TestGame::new(2);
    let gruul = t.battlefield(P0, "Gruul Beastmaster");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(gruul, Entity::Player(P1))]);
    minus_power(&mut t, gruul, 4);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}

// --- beginning of combat triggers -----------------------------------------------------

#[test]
fn jenova_puts_counters_equal_to_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Jenova, Ancient Calamity",
        "If Jenova isn't on the battlefield at the time its first ability resolves, use Jenova's power as it last existed on the battlefield to determine how many counters to put on the target creature."
    );
    supported("Jenova, Ancient Calamity");
    let mut t = TestGame::new(2);
    let jenova = t.battlefield(P0, "Jenova, Ancient Calamity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    beginning_of_combat(&mut t);
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, jenova, 2);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 3);
}

#[test]
fn maester_seymour_puts_counters_equal_to_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Maester Seymour",
        "If Maester Seymour is no longer on the battlefield when its first ability resolves, use its power as it last existed on the battlefield to determine how many +1/+1 counters to put on the target creature."
    );
    supported("Maester Seymour");
    let mut t = TestGame::new(2);
    let seymour = t.battlefield(P0, "Maester Seymour");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    beginning_of_combat(&mut t);
    grow_then_exile(&mut t, seymour, 2);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 3);
}

#[test]
fn ouroboroid_x_is_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Ouroboroid",
        "If Ouroboroid leaves the battlefield before its ability resolves, use its power as it last existed on the battlefield to determine the value of X."
    );
    supported("Ouroboroid");
    let mut t = TestGame::new(2);
    let ouro = t.battlefield(P0, "Ouroboroid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    beginning_of_combat(&mut t);
    grow_then_exile(&mut t, ouro, 2);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 3);
}

// --- enters and leaves triggers ---------------------------------------------------------

#[test]
fn boulderbranch_golem_gains_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Boulderbranch Golem",
        "If Boulderbranch Golem leaves the battlefield before its enters-the-battlefield triggered ability resolves, use its power as it last existed on the battlefield to determine how much life is gained, not its power in the zone it moved to."
    );
    supported("Boulderbranch Golem");
    let mut t = TestGame::new(2);
    let golem = t.enter(P0, "Boulderbranch Golem");
    t.settle();
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, golem, 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
}

#[test]
fn prime_speaker_zegana_draws_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Prime Speaker Zegana",
        "If Prime Speaker Zegana is no longer on the battlefield when its last ability resolves, use its power as it last existed on the battlefield to determine how many cards to draw."
    );
    supported("Prime Speaker Zegana");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest"; 6]);
    let zegana = t.enter(P0, "Prime Speaker Zegana");
    t.settle();
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, zegana, 2);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn prime_speaker_zegana_ignores_creatures_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Prime Speaker Zegana",
        "If Prime Speaker Zegana enters at the same time as another creature you control, you won't consider that creature when determining the greatest power among creatures you control."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let ids = enter_together(&mut t, &[(P0, "Prime Speaker Zegana"), (P0, "Hill Giant")]);
    assert_eq!(plus1(&t, ids[0]), 2, "only the Bears count");
}

#[test]
fn threefold_thunderhulk_creates_gnomes_equal_to_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Threefold Thunderhulk",
        "If Threefold Thunderhulk leaves the battlefield while its triggered ability is on the stack, use its power as it last existed on the battlefield to determine how many Gnomes to create."
    );
    supported("Threefold Thunderhulk");
    let mut t = TestGame::new(2);
    let hulk = t.enter(P0, "Threefold Thunderhulk");
    t.settle();
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, hulk, 2);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Gnome"), 5);
}

#[test]
fn flametongue_yearling_damage_is_its_power_on_resolution() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Flametongue Yearling",
        "Flametongue Yearling's last triggered ability checks its power when it resolves. If it isn't on the battlefield when it resolves, its last known information will be used."
    );
    supported("Flametongue Yearling");
    let mut t = TestGame::new(2);
    let air = t.battlefield(P1, "Air Elemental");
    t.answer_targets(P0, &[obj(air)]);
    let yearling = t.enter(P0, "Flametongue Yearling");
    t.settle();
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, yearling, 1);
    t.resolve_all();
    assert_eq!(damage(&t, air), 3);
}

#[test]
fn flametongue_yearling_zero_power_deals_no_damage() {
    cr!("120.8", "113.7a");
    ruling!(
        "Flametongue Yearling",
        "This may result in no damage being dealt if it had 0 or less power when it left the battlefield."
    );
    let mut t = TestGame::new(2);
    let air = t.battlefield(P1, "Air Elemental");
    t.answer_targets(P0, &[obj(air)]);
    let yearling = t.enter(P0, "Flametongue Yearling");
    t.settle();
    minus_power(&mut t, yearling, 2);
    exile(&mut t, yearling);
    t.resolve_all();
    assert_eq!(damage(&t, air), 0);
}

/// `name` enters under P0's control targeting P1's Air Elemental (4/4); in response it
/// gets `n` +1/+1 counters and is exiled. Returns the damage marked on the Elemental.
fn etb_damage_to_flyer(name: &str, others: usize, n: u32) -> u32 {
    let mut t = TestGame::new(2);
    for _ in 0..others {
        t.battlefield(P0, "Grizzly Bears");
    }
    let air = t.battlefield(P1, "Air Elemental");
    t.answer_targets(P0, &[obj(air)]);
    let src = t.enter(P0, name);
    t.settle();
    assert_eq!(stack(&t), 1, "{name}");
    grow_then_exile(&mut t, src, n);
    t.resolve_all();
    damage(&t, air)
}

#[test]
fn chainweb_aracnir_deals_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Chainweb Aracnir",
        "If Chainweb Aracnir leaves the battlefield before its triggered ability resolves, use its power as it last existed on the battlefield to determine how much damage it deals."
    );
    supported("Chainweb Aracnir");
    assert_eq!(etb_damage_to_flyer("Chainweb Aracnir", 0, 2), 3);
}

#[test]
fn sharpshooter_elf_deals_its_last_known_power() {
    cr!("608.2h", "113.7a", "604.3");
    ruling!(
        "Sharpshooter Elf",
        "If Sharpshooter Elf leaves the battlefield while its triggered ability is on the stack, use its power as it last existed on the battlefield to determine how much damage it deals."
    );
    supported("Sharpshooter Elf");
    // Two Bears and the Elf: power 3 on the battlefield (2 in exile); no counters.
    assert_eq!(etb_damage_to_flyer("Sharpshooter Elf", 2, 0), 3);
}

// --- dies triggers ----------------------------------------------------------------------

/// `name` with `n` +1/+1 (or, if negative, -1/-1) counters is destroyed under P0's
/// control, its dies trigger targeting P1; returns the game afterwards.
fn dies_targeting_p1(name: &str, n: i32) -> TestGame {
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    if n > 0 {
        give_plus1(&mut t, c, n as u32);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    if n < 0 {
        give_minus1(&mut t, c, (-n) as u32);
    } else {
        destroy(&mut t, c);
    }
    assert!(!t.on_battlefield(c), "{name}");
    t.resolve_all();
    t
}

#[test]
fn dreadhorde_butcher_uses_its_last_known_power() {
    cr!("603.10a", "608.2h", "120.8");
    ruling!(
        "Dreadhorde Butcher",
        "Dreadhorde Butcher’s last known existence on the battlefield is checked to determine its power as its last ability resolves. If its power was 0 or less, the ability resolves with no effect."
    );
    supported("Dreadhorde Butcher");
    assert_eq!(lost(&dies_targeting_p1("Dreadhorde Butcher", 2), P1), 3);
    assert_eq!(lost(&dies_targeting_p1("Dreadhorde Butcher", -1), P1), 0);
}

#[test]
fn fireblade_charger_uses_its_last_known_power() {
    cr!("603.10a", "608.2h", "120.8");
    ruling!(
        "Fireblade Charger",
        "Fireblade Charger’s last known existence on the battlefield is checked to determine its power. If its power was 0 or negative, its ability has no effect."
    );
    supported("Fireblade Charger");
    assert_eq!(lost(&dies_targeting_p1("Fireblade Charger", 3), P1), 4);
    assert_eq!(lost(&dies_targeting_p1("Fireblade Charger", -1), P1), 0);
}

#[test]
fn juri_uses_its_last_known_power() {
    cr!("603.10a", "608.2h", "120.8");
    ruling!(
        "Juri, Master of the Revue",
        "For Juri's second ability, use its power from when it was last on the battlefield to determine how much damage is dealt. If that power was 0 or less, Juri deals no damage."
    );
    supported("Juri, Master of the Revue");
    assert_eq!(lost(&dies_targeting_p1("Juri, Master of the Revue", 2), P1), 3);
    assert_eq!(lost(&dies_targeting_p1("Juri, Master of the Revue", -1), P1), 0);
}

#[test]
fn nadier_creates_tokens_equal_to_its_last_known_power() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Nadier, Agent of the Duskenel",
        "For Nadier's second ability, use its power from when it was last on the battlefield to determine how many tokens to create."
    );
    supported("Nadier, Agent of the Duskenel");
    let mut t = TestGame::new(2);
    let nadier = with_counters(&mut t, P0, "Nadier, Agent of the Duskenel", 2);
    exile(&mut t, nadier);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Elf"), 5);
}

#[test]
fn syr_ginger_gains_its_last_known_power() {
    cr!("608.2h", "113.7a", "602.2");
    ruling!(
        "Syr Ginger, the Meal Ender",
        "For Syr Ginger's last ability, use its power from when it was last on the battlefield to determine how much life is gained. If that power was 0 or less, you gain no life."
    );
    supported("Syr Ginger, the Meal Ender");
    let mut t = TestGame::new(2);
    let ginger = with_counters(&mut t, P0, "Syr Ginger, the Meal Ender", 2);
    t.lands(P0, "Plains", 2);
    t.activate(P0, ginger, 0, &[]).unwrap();
    assert!(!t.on_battlefield(ginger), "sacrificed as a cost");
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    // With power 0 or less: no life.
    let mut t = TestGame::new(2);
    let ginger = t.battlefield(P0, "Syr Ginger, the Meal Ender");
    minus_power(&mut t, ginger, 4);
    t.lands(P0, "Plains", 2);
    t.activate(P0, ginger, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

// --- activated and other triggers ---------------------------------------------------------

#[test]
fn spikeshot_elder_deals_its_power_on_resolution() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Spikeshot Elder",
        "How much damage Spikeshot Elder deals as a result of its ability is determined by its power as that ability resolves. If Spikeshot Elder is no longer on the battlefield at that time, its last existence on the battlefield is checked to determine its power."
    );
    supported("Spikeshot Elder");
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Spikeshot Elder");
    t.lands(P0, "Mountain", 3);
    t.activate(P0, elder, 0, &[Entity::Player(P1)]).unwrap();
    // A pump in response counts.
    giant_growth(&mut t, elder);
    t.resolve_all();
    assert_eq!(lost(&t, P1), 4);
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Spikeshot Elder");
    t.lands(P0, "Mountain", 3);
    t.activate(P0, elder, 0, &[Entity::Player(P1)]).unwrap();
    grow_then_exile(&mut t, elder, 2);
    t.resolve_all();
    assert_eq!(lost(&t, P1), 3);
}

#[test]
fn rust_harvester_gets_no_counter_but_deals_its_last_known_power() {
    cr!("608.2h", "113.7a", "608.2b");
    ruling!(
        "Rust Harvester",
        "If Rust Harvester leaves the battlefield before its last ability resolves, it won’t have a +1/+1 counter put on it when the ability resolves. Use its power as it last existed on the battlefield to determine how much damage is dealt."
    );
    supported("Rust Harvester");
    let mut t = TestGame::new(2);
    let harvester = t.battlefield(P0, "Rust Harvester");
    t.graveyard(P0, "Ornithopter");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, harvester, 0, &[Entity::Player(P1)]).unwrap();
    grow_then_exile(&mut t, harvester, 2);
    t.resolve_all();
    assert_eq!(lost(&t, P1), 3);
}

#[test]
fn caldera_pyremaw_deals_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Caldera Pyremaw",
        "If Caldera Pyremaw leaves the battlefield in response to its ability, use its power the last time it existed on the battlefield to determine how much damage it deals as its ability resolves."
    );
    supported("Caldera Pyremaw");
    let mut t = TestGame::new(2);
    let pyremaw = t.battlefield(P0, "Caldera Pyremaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    assert_eq!(stack(&t), 2);
    grow_then_exile(&mut t, pyremaw, 2);
    t.resolve_all();
    assert_eq!(lost(&t, P1), 5);
}

#[test]
fn gau_deals_its_last_known_power() {
    cr!("608.2h", "113.7a", "603.4");
    ruling!(
        "Gau, Feral Youth",
        "If Gau is no longer on the battlefield when its last ability resolves, use its power as it last existed on the battlefield to determine how much damage the ability deals."
    );
    supported("Gau, Feral Youth");
    let mut t = TestGame::new(2);
    let gau = t.battlefield(P0, "Gau, Feral Youth");
    let card = t.graveyard(P0, "Forest");
    exile(&mut t, card);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, gau, 2);
    t.resolve_all();
    assert_eq!(lost(&t, P1), 4);
}

#[test]
fn gau_checks_its_condition_as_the_end_step_begins() {
    cr!("603.4");
    ruling!(
        "Gau, Feral Youth",
        "Gau's last ability checks at the moment it would trigger to see if a card left your graveyard this turn. If none did, the ability won't trigger at all. Once your end step begins, it's too late to make a card leave your graveyard in order to cause this ability to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gau, Feral Youth");
    let card = t.graveyard(P0, "Forest");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(stack(&t), 0);
    exile(&mut t, card);
    assert_eq!(stack(&t), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn vincent_checks_its_last_known_power() {
    cr!("608.2h", "113.7a", "603.4");
    ruling!(
        "Vincent, Vengeful Atoner",
        "If Vincent isn't on the battlefield when its last ability resolves, use its power as it last existed on the battlefield to determine whether it deals damage to each other opponent."
    );
    supported("Vincent, Vengeful Atoner");
    let mut t = TestGame::new(3);
    let vincent = with_counters(&mut t, P0, "Vincent, Vengeful Atoner", 4);
    attack_with(&mut t, &[(vincent, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(lost(&t, P1), 7);
    assert_eq!(stack(&t), 2);
    exile(&mut t, vincent);
    t.resolve_all();
    assert_eq!(lost(&t, P2), 7);
}
