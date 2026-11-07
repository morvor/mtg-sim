//! Rulings batch P080 — costs that depend on attacking creatures: cost reductions locked in
//! before paying (CR 601.2f–h) that don't change the mana value (CR 202.3), the Traps'
//! alternative costs (CR 118.9) and Stone Idol Trap's mandatory reduction, tokens created
//! during combat that can (or can't) block, Qasali Ambusher's free cast, and Batwing
//! Brume's "if {W}/{B} was spent".

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported, tokens};
use crate::r_s08_common::{legal_cast_methods, mana_value};
use crate::r_s21_common::{go_to, legal_blocks};
use crate::r_s33_common::two_headed_giant;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn alternatives(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    legal_cast_methods(t, p, card)
        .into_iter()
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .collect()
}

/// `p`'s creatures `names` attack `target` (in `p`'s turn, declare attackers step).
fn attack(t: &mut TestGame, p: PlayerId, names: &[&str], target: Entity) -> Vec<ObjectId> {
    let ids: Vec<ObjectId> = names.iter().map(|n| t.battlefield(p, n)).collect();
    t.g.combat = None;
    t.set_step(p, Step::BeginningOfCombat);
    let decl: Vec<(ObjectId, Entity)> = ids.iter().map(|i| (*i, target)).collect();
    attack_with(t, &decl);
    t.resolve_all();
    ids
}

#[test]
fn ancient_stone_idol_can_cost_zero() {
    cr!("601.2f");
    ruling!(
        "Ancient Stone Idol",
        "If there are enough attacking creatures, Ancient Stone Idol's cost can be reduced to {0}."
    );
    supported("Ancient Stone Idol");
    let mut t = TestGame::new(2);
    attack(&mut t, P1, &["Memnite"; 10], Entity::Player(P0));
    let idol = t.hand(P0, "Ancient Stone Idol");
    let spell = t.cast(P0, idol).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(mana_value(&t, spell), 10);
}

#[test]
fn ancient_stone_idol_total_cost_is_locked_before_mana_abilities() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Ancient Stone Idol",
        "The total cost to cast a spell is locked in before you pay that cost. For example, if you control five attacking creatures, including one you can sacrifice to add {C}, Ancient Stone Idol costs {5} to cast."
    );
    // P0 attacks with four Memnites and a Blood Pet ("Sacrifice this creature: Add {B}."),
    // then casts the Idol for {5} with four Wastes and the Blood Pet.
    let mut t = TestGame::new(2);
    let ids = attack(
        &mut t,
        P0,
        &["Memnite", "Memnite", "Memnite", "Memnite", "Blood Pet"],
        Entity::Player(P1),
    );
    t.lands(P0, "Wastes", 4);
    let idol = t.hand(P0, "Ancient Stone Idol");
    let spell = t.cast(P0, idol).try_go().expect("cast for {5}");
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(!t.on_battlefield(ids[4]), "the Blood Pet was sacrificed for mana");
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
}

#[test]
fn cost_reductions_dont_change_mana_value() {
    cr!("202.3", "601.2f");
    ruling!(
        "Ephara's Dispersal",
        "The cost reduction ability of Ephara's Dispersal doesn't affect its mana cost or mana value. It affects only the total cost you pay. Specifically, its mana value is always 3."
    );
    ruling!(
        "Sentry Bot",
        "The cost-reduction ability of Sentry Bot doesn’t change its mana cost or mana value, only the total cost you pay. Specifically, the mana value of Sentry Bot is always 5."
    );
    supported("Ephara's Dispersal");
    supported("Sentry Bot");
    let mut t = TestGame::new(2);
    let ids = attack(&mut t, P1, &["Grizzly Bears", "Memnite"], Entity::Player(P0));
    // Ephara's Dispersal for {U}.
    t.lands(P0, "Island", 1);
    let d = t.hand(P0, "Ephara's Dispersal");
    let spell = t.cast(P0, d).target(obj(ids[0])).go();
    assert_eq!(mana_value(&t, spell), 3);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Sentry Bot for {3}{W} with one creature attacking P0.
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    let bot = t.hand(P0, "Sentry Bot");
    let spell = t.cast(P0, bot).try_go().expect("cast for {3}{W}");
    assert_eq!(mana_value(&t, spell), 5);
    t.resolve_all();
    let bot = t.named_on_battlefield("Sentry Bot")[0];
    assert_eq!(mana_value(&t, bot), 5);
    assert_eq!(crate::r_s28_common::energy(&t, P0), 1);
}

#[test]
fn sentry_bot_cost_cant_be_changed_while_it_is_being_cast() {
    cr!("601.2", "601.2f", "601.2h");
    ruling!(
        "Sentry Bot",
        "Once you announce that you’re casting Sentry Bot, no player may take actions until you’re done casting it. Notably, opponents can’t remove creatures from combat to try to change Sentry Bot’s total cost."
    );
    let mut t = TestGame::new(2);
    let ids = attack(&mut t, P1, &["Grizzly Bears", "Memnite"], Entity::Player(P0));
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    let bot = t.hand(P0, "Sentry Bot");
    let from = t.asked().len();
    let spell = t.cast(P0, bot).try_go().expect("cast for {2}{W}");
    assert!(
        t.asked()[from..].iter().all(|(p, _)| *p == P0),
        "no other player acted during the casting"
    );
    // Removing an attacker afterwards doesn't change what was paid.
    mtg_engine::combat::remove_from_combat(&mut t.g, ids[0]);
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sentry Bot").len(), 1);
}

#[test]
fn stone_idol_trap_has_a_mandatory_reduction_counting_any_attackers() {
    cr!("601.2f", "118.9", "508.1");
    ruling!(
        "Stone Idol Trap",
        "This Trap doesn’t work the same way as other Traps. Rather than having an optional alternative cost, Stone Idol Trap has a mandatory cost-reduction ability."
    );
    ruling!(
        "Stone Idol Trap",
        "The cost-reduction ability doesn’t care who controls the attacking creatures. If you like, you may cast Stone Idol Trap during your own combat phase, in which case it will count your attacking creatures. If you do, however, the token you put onto the battlefield will be unable to attack that combat and will be exiled later that turn."
    );
    supported("Stone Idol Trap");
    // P0's own two attackers: {3}{R}.
    let mut t = TestGame::new(2);
    attack(&mut t, P0, &["Grizzly Bears", "Memnite"], Entity::Player(P1));
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let trap = t.hand(P0, "Stone Idol Trap");
    assert!(alternatives(&mut t, P0, trap).is_empty(), "no alternative cost");
    let from = t.asked().len();
    t.cast(P0, trap).try_go().expect("cast for {3}{R}");
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        Decision::ChooseCastingMethod { .. } | Decision::OptionalCost { .. }
    )));
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    assert!(!crate::r_s10_common::attacking(&t, token));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(token), "exiled at this turn's end step");
}

#[test]
fn tokens_from_traps_cast_in_the_declare_attackers_step_can_block() {
    cr!("509.1a");
    ruling!(
        "Stone Idol Trap",
        "If you cast Stone Idol Trap during your opponent’s declare attackers step, the Construct token you put onto the battlefield will be able to block that turn."
    );
    ruling!(
        "Nemesis Trap",
        "If you cast Nemesis Trap during your opponent's declare attackers step, you'll put the token onto the battlefield before the declare blockers step begins and you can block with it this combat. If you cast Nemesis Trap later in the combat phase a, you won't be able to block with the token."
    );
    supported("Nemesis Trap");
    // Stone Idol Trap.
    let mut t = TestGame::new(2);
    let ids = attack(&mut t, P1, &["Grizzly Bears"], Entity::Player(P0));
    cast_card(&mut t, P0, "Stone Idol Trap", &[]).unwrap();
    t.resolve_all();
    let construct = tokens(&t, P0)[0];
    assert!(legal_blocks(&mut t, P0, &[(construct, ids[0])]));
    // Nemesis Trap in the declare attackers step: P0's copy of the exiled Giant can block
    // the Bears.
    let mut t = TestGame::new(2);
    let ids = attack(&mut t, P1, &["Grizzly Bears", "Hill Giant"], Entity::Player(P0));
    cast_card(&mut t, P0, "Nemesis Trap", &[obj(ids[1])]).unwrap();
    t.resolve_all();
    let copy = tokens(&t, P0)[0];
    assert!(legal_blocks(&mut t, P0, &[(copy, ids[0])]));
    crate::r_s01_common::block_and_finish(&mut t, P0, &[(copy, ids[0])]);
    assert!(!t.on_battlefield(ids[0]));
    // In the declare blockers step: too late to block.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s03_common::to_blockers(
        &mut t,
        &[(bears, Entity::Player(P0)), (giant, Entity::Player(P0))],
        &[],
    );
    cast_card(&mut t, P0, "Nemesis Trap", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P0), 18, "the Bears stayed unblocked");
}

#[test]
fn trap_alternative_costs_can_target_any_attacker() {
    cr!("118.9", "601.2b");
    ruling!(
        "Nemesis Trap",
        "If you cast Nemesis Trap for {B}{B}, you may still target any attacking creature, not just the white attacking creature."
    );
    ruling!(
        "Slingbow Trap",
        "If you cast Slingbow Trap for {G}, you may target any attacking creature with flying, not just a black one."
    );
    supported("Slingbow Trap");
    // Nemesis Trap: a white Savannah Lions and a green-less Grizzly Bears attack; {B}{B}
    // exiles the Bears.
    let mut t = TestGame::new(2);
    let ids = attack(
        &mut t,
        P1,
        &["Savannah Lions", "Grizzly Bears"],
        Entity::Player(P0),
    );
    t.lands(P0, "Swamp", 2);
    let trap = t.hand(P0, "Nemesis Trap");
    let alt = alternatives(&mut t, P0, trap);
    assert_eq!(alt.len(), 1);
    t.cast(P0, trap).method(alt[0].clone()).target(obj(ids[1])).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Slingbow Trap: black Vampire Bats and white Serra Angel attack; {G} destroys the
    // Angel.
    let mut t = TestGame::new(2);
    let ids = attack(
        &mut t,
        P1,
        &["Vampire Bats", "Serra Angel"],
        Entity::Player(P0),
    );
    t.lands(P0, "Forest", 1);
    let trap = t.hand(P0, "Slingbow Trap");
    let alt = alternatives(&mut t, P0, trap);
    assert_eq!(alt.len(), 1);
    t.cast(P0, trap).method(alt[0].clone()).target(obj(ids[1])).go();
    t.resolve_all();
    assert!(!t.on_battlefield(ids[1]));
    assert!(t.on_battlefield(ids[0]));
}

#[test]
fn nemesis_trap_token_copies_only_the_printed_values() {
    cr!("707.2", "111.3", "613.1a");
    ruling!(
        "Nemesis Trap",
        "The token you put onto the battlefield copies exactly what was printed on the exiled creature and nothing more"
    );
    // P1's Grizzly Bears (tapped, with a +1/+1 counter and Holy Strength) and an animated
    // Mutavault attack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s13_common::add(&mut t, bears, "+1/+1", 1);
    crate::r_s06_common::attach_new(&mut t, P1, "Holy Strength", bears);
    let vault = t.battlefield(P1, "Mutavault");
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    add(&mut t, P1, mtg_engine::mana::ManaType::C, 1);
    crate::r_s06_common::activate_containing(&mut t, P1, vault, "becomes").expect("animated");
    t.resolve_all();
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P0)), (vault, Entity::Player(P0))],
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 5));
    for target in [bears, vault] {
        cast_card(&mut t, P0, "Nemesis Trap", &[obj(target)]).unwrap();
        t.resolve_all();
    }
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    let bear_copy = *toks
        .iter()
        .find(|x| t.obj_now(**x).chars.name == "Grizzly Bears")
        .unwrap();
    assert_eq!(t.pt(bear_copy), (2, 2));
    assert!(!t.obj_now(bear_copy).tapped);
    assert_eq!(t.counters(bear_copy, "+1/+1"), 0);
    let vault_copy = *toks
        .iter()
        .find(|x| t.obj_now(**x).chars.name == "Mutavault")
        .unwrap();
    assert!(!crate::r_s04_common::is_creature(&t, vault_copy), "just a land");
}

/// P1 has Qasali Ambusher in hand and controls `lands`.
fn qasali(t: &mut TestGame, lands: &[&str]) -> ObjectId {
    for l in lands {
        t.battlefield(P1, l);
    }
    t.hand(P1, "Qasali Ambusher")
}

#[test]
fn qasali_ambusher_can_be_cast_through_end_of_combat_but_wont_block() {
    cr!("601.3c", "509.1a", "506.4");
    ruling!(
        "Qasali Ambusher",
        "If you want to cast Qasali Ambusher by using its second ability, you don’t necessarily have to do it during the declare attackers step. You can do it later in combat (including all the way up through the end of combat step) as long as there’s still a creature attacking you."
    );
    let mut t = TestGame::new(2);
    let q = qasali(&mut t, &["Forest", "Plains"]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    go_to(&mut t, Step::EndOfCombat);
    let alt = alternatives(&mut t, P1, q);
    assert_eq!(alt.len(), 1);
    t.cast(P1, q).method(alt[0].clone()).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Qasali Ambusher").len(), 1);
    assert_eq!(t.life(P1), 18, "too late to block");
}

#[test]
fn qasali_ambusher_lands_need_the_subtypes_not_the_names() {
    cr!("205.3i", "305.6");
    ruling!(
        "Qasali Ambusher",
        "Qasali Ambusher’s second ability is looking for lands with the subtypes Forest and Plains. Those lands don’t need to be named Forest and Plains. It’s okay if both subtypes are from the same land"
    );
    supported("Savannah");
    let mut t = TestGame::new(2);
    let q = qasali(&mut t, &["Savannah"]);
    attack(&mut t, P0, &["Grizzly Bears"], Entity::Player(P1));
    assert_eq!(alternatives(&mut t, P1, q).len(), 1);
    // Without a Plains: no.
    let mut t = TestGame::new(2);
    let q = qasali(&mut t, &["Forest", "Forest"]);
    attack(&mut t, P0, &["Grizzly Bears"], Entity::Player(P1));
    assert!(alternatives(&mut t, P1, q).is_empty());
}

#[test]
fn qasali_ambusher_two_headed_giant_teammate_attacked_isnt_enough() {
    cr!("810.7", "506.2");
    ruling!(
        "Qasali Ambusher",
        "If you’re playing a Two-Headed Giant game and a creature is attacking your teammate, you won’t be able to cast Qasali Ambusher this way."
    );
    for (target, ok) in [(P0, false), (P1, true)] {
        let mut t = two_headed_giant();
        let q = qasali(&mut t, &["Forest", "Plains"]);
        attack(&mut t, P2, &["Grizzly Bears"], Entity::Player(target));
        assert_eq!(alternatives(&mut t, P1, q).len(), ok as usize, "{target}");
    }
}

#[test]
fn batwing_brume_checks_which_colors_were_spent() {
    cr!("601.2h", "608.2h", "615.1");
    ruling!(
        "Batwing Brume",
        "The spell checks on resolution to see if any mana of the stated colors was spent to pay its cost. If so, it doesn’t matter how much mana of that color was spent."
    );
    supported("Batwing Brume");
    // (lands, P0's life, P1's life) after P0's Grizzly Bears attack P1.
    for (lands, p0, p1) in [
        (["Plains", "Plains"], 20, 20),
        (["Swamp", "Swamp"], 19, 18),
        (["Plains", "Swamp"], 19, 20),
    ] {
        let mut t = TestGame::new(2);
        attack(&mut t, P0, &["Grizzly Bears"], Entity::Player(P1));
        for l in lands {
            t.battlefield(P1, l);
        }
        let brume = t.hand(P1, "Batwing Brume");
        t.cast(P1, brume).go();
        t.resolve_all();
        go_to(&mut t, Step::EndOfCombat);
        assert_eq!((t.life(P0), t.life(P1)), (p0, p1), "{lands:?}");
    }
}
