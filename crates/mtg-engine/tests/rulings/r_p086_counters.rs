//! Rulings batch P086 — hate for infect, wither and -1/-1 counters: Melira, Sylvok
//! Outcast, Melira, the Living Cure, Vizier of Remedies; and Unable to Scream's layers.

use crate::r_p086_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn poison(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::POISON)
}

fn has_kw(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    crate::r_s06_common::has_kw(t, id, kw)
}

// --- Melira, Sylvok Outcast -------------------------------------------------------------------

#[test]
fn melira_doesnt_stop_damage_removing_loyalty_counters() {
    cr!("120.3c", "306.8");
    ruling!(
        "Melira, Sylvok Outcast",
        "Damage dealt to planeswalkers you control causes loyalty counters to be removed from them, regardless of whether Melira is on the battlefield."
    );
    supported("Melira, Sylvok Outcast");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Melira, Sylvok Outcast");
    let gideon = t.battlefield(P0, "Gideon, Ally of Zendikar");
    cast_new(&mut t, P1, "Shock", &[obj(gideon)]);
    t.resolve_all();
    assert_eq!(t.counters(gideon, counters::LOYALTY), 2);
}

#[test]
fn melira_removes_infect_from_creatures_entering_later() {
    cr!("611.3a", "613.1f");
    ruling!(
        "Melira, Sylvok Outcast",
        "If a creature with infect enters under an opponent's control after Melira enters, that creature loses infect."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Melira, Sylvok Outcast");
    let mamba = t.enter(P1, "Blight Mamba");
    t.settle();
    assert!(!has_kw(&t, mamba, KeywordKind::Infect));
    // Its own controller's creatures keep infect.
    let mine = t.enter(P0, "Blight Mamba");
    assert!(has_kw(&t, mine, KeywordKind::Infect));
}

#[test]
fn melira_doesnt_remove_counters_already_there() {
    cr!("611.3a", "122.1");
    ruling!(
        "Melira, Sylvok Outcast",
        "Melira doesn't remove any poison counters you already have or any -1/-1 counters already on creatures you control."
    );
    let mut t = TestGame::new(2);
    t.g.add_counters(Entity::Player(P0), counters::POISON, 2, None);
    let wurm = t.battlefield(P0, "Craw Wurm");
    put_counters(&mut t, wurm, counters::MINUS1, 1);
    t.enter(P0, "Melira, Sylvok Outcast");
    t.settle();
    assert_eq!(poison(&t, P0), 2);
    assert_eq!(t.counters(wurm, counters::MINUS1), 1);
    // New ones can't be put.
    put_counters(&mut t, wurm, counters::MINUS1, 1);
    assert_eq!(t.counters(wurm, counters::MINUS1), 1);
}

#[test]
fn melira_and_a_creature_that_gains_infect_later() {
    cr!("613.7", "120.3b", "120.3d", "702.15b");
    ruling!(
        "Melira, Sylvok Outcast",
        "It's possible for a creature an opponent controls to gain infect after Melira enters. For example, your opponent could cast Glistening Oil on a creature, and then Melira's ability and Glistening Oil's ability would be applied in timestamp order, with the most recent effect winning. The damage dealt to you by such a creature won't result in you losing life (because the creature has infect) or in you getting poison counters (because Melira stops them). Similarly, damage dealt to a creature you control by such a creature won't result in damage being marked on the creature or in -1/-1 counters. However, abilities that check whether damage was dealt (such as lifelink or Whispering Specter's triggered ability) will still see that damage."
    );
    // Child of Night: a 2/1 with lifelink.
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Melira, Sylvok Outcast");
        let child = t.battlefield(P1, "Child of Night");
        attach_new(t, P1, "Glistening Oil", obj(child));
        assert!(has_kw(t, child, KeywordKind::Infect));
        child
    };
    // To a player.
    let mut t = TestGame::new(2);
    let child = setup(&mut t);
    t.set_step(P1, mtg_engine::turn::Step::BeginningOfCombat);
    attack_with(&mut t, &[(child, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(poison(&t, P0), 0);
    assert_eq!(t.life(P1), 22);
    // To a creature.
    let mut t = TestGame::new(2);
    let child = setup(&mut t);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.set_step(P1, mtg_engine::turn::Step::BeginningOfCombat);
    attack_with(&mut t, &[(child, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wurm, child)]),
    );
    t.advance_to(P1, mtg_engine::turn::Step::EndOfCombat);
    t.settle();
    assert!(t.on_battlefield(wurm));
    assert_eq!(damage_marked(&t, wurm), 0);
    assert_eq!(t.counters(wurm, counters::MINUS1), 0);
    assert_eq!(t.life(P1), 22);
}

// --- Melira, the Living Cure ------------------------------------------------------------------

/// Answers a replacement-order choice with the first option whose text contains
/// "poison counter" and "instead you get one" (Melira's), if any.
fn melira_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| o.to_lowercase().contains("one poison counter"))
            .map(Answer::Index),
        _ => None,
    }
}

#[test]
fn melira_the_living_cure_and_vorinclex_the_receiving_player_chooses_the_order() {
    cr!("616.1", "120.3b", "122.6");
    ruling!(
        "Melira, the Living Cure",
        "Melira's first ability creates a replacement effect. If another replacement effect would also change the number of poison counters a player receives, such as that of Vorinclex, Monstrous Raider, the player receiving the counters chooses which one to apply first, regardless of who controls what permanents. This will usually result in that player choosing to apply Melira's effect last and receive one counter. However, if they want to, they could apply Melira's replacement effect first and receive more counters."
    );
    supported("Melira, the Living Cure");
    for melira_last in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Melira, the Living Cure");
        t.battlefield(P1, "Vorinclex, Monstrous Raider");
        let rats = t.battlefield(P1, "Septic Rats");
        if melira_last {
            crate::r_s03_common::respond(&mut t, P0, crate::r_s29_common::twice_first);
        } else {
            crate::r_s03_common::respond(&mut t, P0, melira_first);
        }
        let from = t.asked().len();
        t.set_step(P1, mtg_engine::turn::Step::BeginningOfCombat);
        attack_with(&mut t, &[(rats, Entity::Player(P0))]);
        block_and_finish(&mut t, P0, &[]);
        assert_eq!(
            crate::r_s29_common::replacement_choosers(&t, from),
            vec![P0],
            "melira_last={melira_last}"
        );
        assert_eq!(poison(&t, P0), if melira_last { 1 } else { 2 });
    }
}

// --- Vizier of Remedies -----------------------------------------------------------------------

#[test]
fn vizier_of_remedies_one_fewer_counter_but_all_the_damage_is_dealt() {
    cr!("120.3d", "702.15b", "614.1a");
    ruling!(
        "Vizier of Remedies",
        "If a creature you control is dealt damage by a source with wither or infect, that much damage is dealt, but one fewer -1/-1 counter is put on your creature. For example, if a 2/2 creature with lifelink and infect is blocked by Vizier of Remedies, the first creature’s controller gains 2 life and puts one -1/-1 counter on Vizier of Remedies."
    );
    supported("Vizier of Remedies");
    let mut t = TestGame::new(2);
    let vizier = t.battlefield(P0, "Vizier of Remedies");
    let rats = t.battlefield(P1, "Septic Rats");
    attach_new(&mut t, P1, "Lifelink", obj(rats));
    t.set_step(P1, mtg_engine::turn::Step::BeginningOfCombat);
    attack_with(&mut t, &[(rats, Entity::Player(P0))]);
    let id = t.g.current(vizier);
    // Watch the counters put on the Vizier as combat damage is dealt.
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(vizier, rats)]),
    );
    t.advance_to(P1, mtg_engine::turn::Step::CombatDamage);
    assert_eq!(t.counters(id, counters::MINUS1), 1);
    assert_eq!(t.life(P1), 22);
    t.settle();
    // A 2/1 with one -1/-1 counter dies.
    assert!(t.in_graveyard(P0, "Vizier of Remedies"));
}

#[test]
fn vizier_of_remedies_one_fewer_counter_for_simultaneous_damage() {
    cr!("120.3d", "510.2", "614.1a");
    ruling!(
        "Vizier of Remedies",
        "If multiple creatures with wither and/or infect deal damage to a creature at once, Vizier of Remedies causes only one counter fewer to be put on that creature. Its effect doesn’t apply separately for each creature dealing damage."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of Remedies");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let a = t.battlefield(P1, "Septic Rats");
    let b = t.battlefield(P1, "Septic Rats");
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(a, wurm), (b, wurm)]);
    assert_eq!(t.counters(wurm, counters::MINUS1), 3);
}

#[test]
fn vizier_of_remedies_doesnt_apply_to_creatures_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Vizier of Remedies",
        "If an effect puts Vizier of Remedies onto the battlefield with one or more -1/-1 counters on it, its effect won’t apply. This is because you must control Vizier of Remedies before the creature begins to enter the battlefield for its effect to apply. The same is true of any creatures entering the battlefield with -1/-1 counters on them at the same time as Vizier of Remedies enters the battlefield."
    );
    let mut t = TestGame::new(2);
    let vizier = t.exile(P0, "Vizier of Remedies");
    let wurm = t.exile(P0, "Craw Wurm");
    let mv = |id| mtg_engine::replacement::MoveEv {
        obj: id,
        to: Zone::Battlefield,
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(P0),
            counters: vec![(counters::MINUS1.into(), 1)],
            ..Default::default()
        },
        source: None,
    };
    let new = t.g.move_objects(vec![mv(vizier), mv(wurm)]);
    t.g.recompute();
    let wurm = new[1].expect("the Wurm entered");
    assert_eq!(t.counters(wurm, counters::MINUS1), 1);
    assert_eq!(t.pt(wurm), (5, 3));
    t.settle();
    // The Vizier itself entered as a 1/0.
    assert!(t.in_graveyard(P0, "Vizier of Remedies"));
    // A creature entering later with -1/-1 counters gets one fewer.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of Remedies");
    let wurm = t.exile(P0, "Craw Wurm");
    let new = t.g.move_object_ev(mv(wurm)).expect("entered");
    t.g.recompute();
    assert_eq!(t.counters(new, counters::MINUS1), 0);
}

// --- Unable to Scream -------------------------------------------------------------------------

#[test]
fn unable_to_scream_keeps_earlier_pt_changes_and_later_abilities() {
    cr!("613.4b", "613.4c", "613.4d", "613.1f");
    ruling!(
        "Unable to Scream",
        "Effects that modify a creature's power and toughness, such as the effect of Jump Scare, will apply to that creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    ruling!(
        "Unable to Scream",
        "If the affected permanent gains an ability after Unable to Scream becomes attached to it, it will keep that ability."
    );
    supported("Unable to Scream");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    attach_new(&mut t, P0, "Unable to Scream", obj(bears));
    // Base 0/2, +1/+1 counter, +3/+3.
    assert_eq!(t.pt(bears), (4, 6));
    // Switched (Twisted Image) after the Aura: still in layer 7d, after everything else.
    cast_new(&mut t, P1, "Twisted Image", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 4));
    // Gaining flying after the Aura: it keeps it (Jump).
    cast_new(&mut t, P1, "Jump", &[obj(bears)]);
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    // An ability gained before the Aura is lost.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P1, "Jump", &[obj(bears)]);
    t.resolve_all();
    let wolf = t.g.current(bears);
    attach_new(&mut t, P0, "Unable to Scream", obj(wolf));
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
    // A switch that began before the Aura still applies.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P1, "Twisted Image", &[obj(bears)]);
    t.resolve_all();
    attach_new(&mut t, P0, "Unable to Scream", obj(bears));
    assert_eq!(t.pt(bears), (2, 0));
}
