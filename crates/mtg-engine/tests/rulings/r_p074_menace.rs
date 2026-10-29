//! Rulings batch P074 — granting menace (and melee): redundant instances (Atarka
//! Pummeler, Goblin War Drums, Caterwauling Boggart), "can't be blocked by more than one
//! creature", tokens that enter attacking (Inquisitorial Rosette), "one of your opponents"
//! attack triggers in multiplayer (Frenzied Gorespawn, Frontier Warmonger), Blood Curdle's
//! untargeted counter, Drach'Nyen's exiled card's power, and Adriana's melee.

use crate::r_s01_common::{attack_with, give_mana_for, supported, triggers_on_stack};
use crate::r_s04_common::add_mana;
use crate::r_s05_common::enter;
use crate::r_s06_common::{activate_containing, attach_new, has_kw};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const MENACE: KeywordKind = KeywordKind::Menace;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn menace_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id).chars.keywords_of(MENACE).count()
}

/// P0's `attacker` attacks P1, who declares `blockers` on it; combat finishes. Returns
/// whether the attacker was blocked (P1 took no damage).
fn blocked_by(t: &mut TestGame, attacker: ObjectId, blockers: &[ObjectId]) -> bool {
    let before = t.life(P1);
    attack_with(t, &[(attacker, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(blockers.iter().map(|b| (*b, attacker)).collect()),
    );
    t.advance_to(P0, Step::PostcombatMain);
    t.life(P1) == before
}

#[test]
fn redundant_menace_still_allows_two_blockers() {
    cr!("702.111a", "702.111b", "509.1b");
    ruling!(
        "Atarka Pummeler",
        "Activating Atarka Pummeler’s ability more than once conveys no additional benefit to creatures you control."
    );
    ruling!(
        "Goblin War Drums",
        "Multiple War Drums are not cumulative. They are redundant."
    );
    ruling!(
        "Caterwauling Boggart",
        "Each creature you control that’s both a Goblin and an Elemental can’t be blocked except by two or more creatures (as opposed to four or more creatures)."
    );
    supported("Atarka Pummeler");
    // Atarka Pummeler, activated twice (Pummeler 4 + Craw Wurm 6 = 8 power).
    let mut t = TestGame::new(2);
    let pummeler = t.battlefield(P0, "Atarka Pummeler");
    let wurm = t.battlefield(P0, "Craw Wurm");
    for _ in 0..2 {
        add_mana(&mut t, P0, ManaType::R, 2);
        add_mana(&mut t, P0, ManaType::C, 3);
        activate_containing(&mut t, P0, pummeler, "menace").unwrap();
        t.resolve_all();
    }
    assert_eq!(menace_count(&t, wurm), 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    assert!(blocked_by(&mut t, wurm, &[a, b]));
    // Two Goblin War Drums; one blocker isn't enough, two are.
    for (blockers, blocked) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Goblin War Drums");
        t.battlefield(P0, "Goblin War Drums");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let bs: Vec<_> = (0..blockers)
            .map(|_| t.battlefield(P1, "Llanowar Elves"))
            .collect();
        assert_eq!(menace_count(&t, bears), 2);
        assert_eq!(blocked_by(&mut t, bears, &bs), blocked, "{blockers} blockers");
    }
    // Caterwauling Boggart: a Goblin Elemental (a changeling) has menace once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Caterwauling Boggart");
    let ch = t.battlefield(P0, "Woodland Changeling");
    assert_eq!(menace_count(&t, ch), 1);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    assert!(blocked_by(&mut t, ch, &[a, b]));
}

#[test]
fn menace_and_cant_be_blocked_by_more_than_one_means_unblockable() {
    cr!("702.111b", "509.1b", "509.1c");
    ruling!(
        "Goblin War Drums",
        "When combined with an effect that reads \"Each creature you control can't be blocked by more than one creature,\" none of your creatures can be blocked."
    );
    supported("Familiar Ground");
    for n in [1, 2] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Goblin War Drums");
        t.battlefield(P0, "Familiar Ground");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let bs: Vec<_> = (0..n).map(|_| t.battlefield(P1, "Llanowar Elves")).collect();
        assert!(!blocked_by(&mut t, bears, &bs), "{n} blockers");
        assert_eq!(t.life(P1), 18);
    }
}

#[test]
fn inquisitorial_rosette_token_enters_attacking_without_attack_triggers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Inquisitorial Rosette",
        "Although the tokens enter the battlefield attacking, they were never declared as attacking creatures. Abilities that trigger whenever a creature attacks won't trigger."
    );
    let mut t = TestGame::new(2);
    // Raid Bombardment: "Whenever a creature you control with power 2 or less attacks,
    // this enchantment deals 1 damage to the player or planeswalker it's attacking."
    t.battlefield(P0, "Raid Bombardment");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Inquisitorial Rosette", obj(bears));
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert_eq!(triggers_on_stack(&t, "deals 1 damage"), 1);
    t.resolve_all();
    let astartes = crate::r_s05_common::tokens_with_subtype(&t, P0, "Astartes");
    assert_eq!(astartes.len(), 1);
    assert!(t.g.is_attacking(astartes[0]));
    assert!(has_kw(&t, astartes[0], MENACE) && has_kw(&t, bears, MENACE));
    assert_eq!(t.stack_len(), 0, "the token's arrival triggered nothing");
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 20 - 1 - 2 - 2);
}

#[test]
fn attacks_on_your_opponents_by_other_players_trigger() {
    cr!("508.1", "603.2", "802.2");
    ruling!(
        "Frenzied Gorespawn",
        "Creatures your opponents control attacking one of your opponents will cause Frenzied Gorespawn's ability to trigger."
    );
    ruling!(
        "Frontier Warmonger",
        "Creatures your opponents control attacking one of your opponents will cause Frontier Warmonger's ability to trigger."
    );
    ruling!(
        "Frenzied Gorespawn",
        "Only creatures attacking your opponents will gain menace until end of turn. Creatures attacking planeswalkers or battles won't gain menace, no matter who controls or protects those permanents."
    );
    supported("Frenzied Gorespawn");
    supported("Frontier Warmonger");
    // (card, whether an attack on P2's planeswalker triggers it)
    for (card, walker_triggers) in [("Frenzied Gorespawn", false), ("Frontier Warmonger", true)]
    {
        let mut t = TestGame::new(3);
        t.battlefield(P0, card);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Llanowar Elves");
        let jace = t.battlefield(P2, "Jace Beleren");
        t.advance_to(P1, Step::PrecombatMain);
        attack_with(&mut t, &[(a, pl(P2)), (b, obj(jace))]);
        t.resolve_all();
        assert!(has_kw(&t, a, MENACE), "{card}");
        assert_eq!(has_kw(&t, b, MENACE), walker_triggers, "{card}");
    }
    // An attack on you doesn't trigger it.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Frenzied Gorespawn");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    attack_with(&mut t, &[(a, pl(P0))]);
    t.resolve_all();
    assert!(!has_kw(&t, a, MENACE));
}

#[test]
fn blood_curdle_counter_is_chosen_after_the_destruction() {
    cr!("608.2c", "115.1", "122.1b");
    ruling!(
        "Blood Curdle",
        "Blood Curdle targets only one creature: the one that will be destroyed. You choose which creature to put a menace counter on after the target creature has been destroyed."
    );
    ruling!(
        "Blood Curdle",
        "If a creature you control was exiled until the creature that was destroyed left the battlefield, the exiled creature returns in time to receive the counter."
    );
    supported("Blood Curdle");
    supported("Banisher Priest");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P1, &[obj(bears)]);
    let priest = enter(&mut t, P1, "Banisher Priest");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    give_mana_for(&mut t, P0, "Blood Curdle");
    let bc = t.hand(P0, "Blood Curdle");
    let from = t.asked().len();
    t.cast(P0, bc).target(priest).go();
    // Only one target was asked for.
    let targets = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count();
    assert_eq!(targets, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Banisher Priest"));
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "menace"), 1);
    assert!(has_kw(&t, back[0], MENACE));
}

#[test]
fn blood_curdle_illegal_target_vs_indestructible_target() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Blood Curdle",
        "If the target creature is an illegal target by the time Blood Curdle tries to resolve, the spell won’t resolve. You won’t put a menace counter on any creature. If the target is legal but not destroyed (most likely because it has indestructible), you do put a menace counter on a creature you control."
    );
    // Illegal: the target leaves first.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    give_mana_for(&mut t, P0, "Blood Curdle");
    let bc = t.hand(P0, "Blood Curdle");
    t.cast(P0, bc).target(elves).go();
    crate::r_s02_common::destroy(&mut t, elves);
    t.resolve_all();
    assert_eq!(t.counters(bears, "menace"), 0);
    // Indestructible: not destroyed, but the counter is put.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let myr = t.battlefield(P1, "Darksteel Myr");
    give_mana_for(&mut t, P0, "Blood Curdle");
    let bc = t.hand(P0, "Blood Curdle");
    t.cast(P0, bc).target(myr).go();
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.counters(bears, "menace"), 1);
}

#[test]
fn moodmark_painter_with_no_creature_cards_gives_only_menace() {
    cr!("608.2h", "702.111a");
    ruling!(
        "Moodmark Painter",
        "If there are no creature cards in your graveyard, the target creature just gains menace until end of turn."
    );
    supported("Moodmark Painter");
    for graveyard in [0, 2] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        for _ in 0..graveyard {
            t.graveyard(P0, "Llanowar Elves");
        }
        t.answer_targets(P0, &[obj(bears)]);
        enter(&mut t, P0, "Moodmark Painter");
        t.resolve_all();
        assert!(has_kw(&t, bears, MENACE));
        assert_eq!(t.pt(bears), (2 + graveyard, 2));
    }
}

#[test]
fn lose_calm_can_target_your_own_untapped_creature() {
    cr!("115.1", "701.26b");
    ruling!(
        "Lose Calm",
        "Lose Calm can target any creature, including one that’s untapped or one you already control."
    );
    supported("Lose Calm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Lose Calm");
    let lc = t.hand(P0, "Lose Calm");
    t.cast(P0, lc).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(has_kw(&t, bears, MENACE) && has_kw(&t, bears, KeywordKind::Haste));
    assert!(!t.obj_now(bears).tapped);
    // And an opponent's untapped creature.
    let elves = t.battlefield(P1, "Llanowar Elves");
    give_mana_for(&mut t, P0, "Lose Calm");
    let lc = t.hand(P0, "Lose Calm");
    t.cast(P0, lc).target(elves).go();
    t.resolve_all();
    assert_eq!(t.obj_now(elves).controller, P0);
}

#[test]
fn drachnyen_uses_the_exiled_cards_power_in_exile() {
    cr!("604.3", "208.3", "400.7");
    ruling!(
        "Drach'Nyen",
        "If the exiled card's power is determined by a characteristic-defining ability, usually denoted by a \"*\" in the power/toughness box, that ability applies in exile and should be used to determine the value of X."
    );
    ruling!(
        "Drach'Nyen",
        "If the exiled card doesn't have a printed power, perhaps because it was a noncreature permanent that was only a creature on the battlefield due to another effect, X is 0."
    );
    supported("Drach'Nyen");
    // Tarmogoyf: in exile, a creature card and an instant in graveyards make it 2/3.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goyf = t.battlefield(P1, "Tarmogoyf");
    t.graveyard(P1, "Llanowar Elves");
    t.graveyard(P1, "Lightning Bolt");
    t.answer_targets(P0, &[obj(goyf)]);
    let dn = enter(&mut t, P0, "Drach'Nyen");
    t.resolve_all();
    assert!(t.in_exile("Tarmogoyf"));
    assert!(t.g.attach(dn, obj(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(has_kw(&t, bears, MENACE));
    t.graveyard(P1, "Lava Spike");
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 2), "a sorcery too");
    // An animated Mutavault: no printed power in exile.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let vault = t.battlefield(P1, "Mutavault");
    add_mana(&mut t, P1, ManaType::C, 1);
    activate_containing(&mut t, P1, vault, "becomes").unwrap();
    t.resolve_all();
    assert!(crate::r_s07_common::creature_on_battlefield(&t, vault));
    t.answer_targets(P0, &[obj(vault)]);
    let dn = enter(&mut t, P0, "Drach'Nyen");
    t.resolve_all();
    assert!(t.in_exile("Mutavault"));
    assert!(t.g.attach(dn, obj(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    assert!(has_kw(&t, bears, MENACE));
}

#[test]
fn drachnyen_copied_trigger_counts_every_exiled_card() {
    cr!("707.10", "607.2a");
    ruling!(
        "Drach'Nyen",
        "If more than one card is exiled by Drach'Nyen, usually because its triggered ability was copied one or more times, X is the total power of the exiled cards."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let resonator = t.battlefield(P0, "Strionic Resonator");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(wurm)]);
    let dn = enter(&mut t, P0, "Drach'Nyen");
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack.last().copied().unwrap();
    add_mana(&mut t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[obj(trig)]);
    activate_containing(&mut t, P0, resonator, "Copy").unwrap();
    // The copy may choose new targets: the Hill Giant.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm") && t.in_exile("Hill Giant"));
    assert!(t.g.attach(dn, obj(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (2 + 6 + 3, 2));
}

#[test]
fn melee_counts_opponents_attacked_not_creatures() {
    cr!("702.121a", "506.2");
    ruling!(
        "Adriana, Captain of the Guard",
        "It doesn't matter how many creatures you attacked a player with, only that you attacked a player with at least one creature."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Adriana, Captain of the Guard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let others: Vec<_> = (0..3).map(|_| t.battlefield(P0, "Llanowar Elves")).collect();
    let mut attacks = vec![(bears, pl(P1))];
    attacks.extend(others.iter().map(|o| (*o, pl(P2))));
    attack_with(&mut t, &attacks);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4), "two opponents attacked");
    assert_eq!(t.pt(others[0]), (3, 3));
}
