//! Rulings batch P005 — "whenever this or another creature dies" triggers: permanents that
//! die together with the source of the ability look back in time (CR 603.10a) and the
//! ability triggers once per creature (CR 603.2c); players losing at the same time
//! (CR 704.3); Two-Headed Giant (CR 810.9).

use crate::r_p044_common::{
    cast_spell, cleansing, dies_with_its_controller, kill, two_headed_giant, wrath,
};
use crate::r_s01_common::{supported, tokens};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn put_plus(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.add_counters(Entity::Object(id), "+1/+1", n, None);
    t.g.recompute();
}

fn clues(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Clue"))
        .count()
}

/// One case: P0 controls `source` and `others` (by name); P1 casts `spell` (a mass
/// removal spell) on P0's turn... cast by P0 here, which doesn't matter for the
/// triggers; afterwards P1 has lost `loss` life and P0 has gained `gain` (from 20).
struct Case {
    source: &'static str,
    others: &'static [&'static str],
    spell: &'static str,
    loss: i32,
    gain: i32,
}

fn run_case(c: &Case) {
    supported(c.source);
    let mut t = TestGame::new(2);
    t.battlefield(P0, c.source);
    for o in c.others {
        if *o == "Saproling" {
            crate::r_s02_common::create_token(&mut t, P0, "Saproling");
        } else {
            t.battlefield(P0, o);
        }
    }
    // "Target player" / "target opponent": P1 for every trigger.
    for _ in 0..4 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    cast_spell(&mut t, P0, c.spell, &[]);
    t.clear_answers();
    assert!(t.named_on_battlefield(c.source).is_empty(), "{}", c.source);
    assert_eq!(t.life(P1), 20 - c.loss, "{}: opponent's loss", c.source);
    assert_eq!(t.life(P0), 20 + c.gain, "{}: controller's gain", c.source);
}

#[test]
fn dies_together_with_the_source_triggers_for_each() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Blood Artist",
        "If Blood Artist and one or more other creatures die at the same time, its ability will trigger for each of those creatures."
    );
    ruling!(
        "Falkenrath Noble",
        "If Falkenrath Noble and another creature die at the same time, Falkenrath Noble's triggered ability will trigger for each of them."
    );
    ruling!(
        "Vengeful Bloodwitch",
        "If Vengeful Bloodwitch dies at the same time as one or more other creatures you control, its ability will trigger for each of those creatures, including itself."
    );
    ruling!(
        "Vindictive Vampire",
        "If Vindictive Vampire dies at the same time as one or more other creatures you control, Vindictive Vampire’s ability triggers for each of those other creatures."
    );
    ruling!(
        "Zulaport Cutthroat",
        "If another creature you control dies at the same time as Zulaport Cutthroat, its ability triggers for that creature."
    );
    ruling!(
        "Slimefoot, the Stowaway",
        "If one or more Saprolings you control die at the same time as Slimefoot, Slimefoot's first ability will still trigger once for each Saproling that died."
    );
    ruling!(
        "Elas il-Kor, Sadistic Pilgrim",
        "If one or more other creatures you control die at the same time that Elas il-Kor dies, its last ability triggers for each of those other creatures."
    );
    ruling!(
        "Al Bhed Salvagers",
        "If Al Bhed Salvagers dies at the same time as one or more other creatures and/or artifacts you control, its ability will trigger once for each of those permanents, including itself."
    );
    ruling!(
        "Susurian Voidborn",
        "If Susurian Voidborn dies at the same time as one or more creatures or artifacts you control, its first ability will still trigger for itself and each of those creatures and artifacts."
    );
    ruling!(
        "Bastion of Remembrance",
        "If one or more creatures you control die at the same time that Bastion of Remembrance leaves the battlefield, its last ability triggers for each of those creatures."
    );
    ruling!(
        "Cauldron of Essence",
        "If one or more creatures you control die at the same time that Cauldron of Essence leaves the battlefield, its first ability triggers once for each of those creatures."
    );
    ruling!(
        "Rakish Crew",
        "If one or more outlaws you control die at the same time that Rakish Crew leaves the battlefield, its last ability triggers for each of those outlaws."
    );
    let two = &["Grizzly Bears", "Hill Giant"][..];
    let cases = [
        // Itself and two others: three triggers.
        Case {
            source: "Blood Artist",
            others: two,
            spell: "Wrath of God",
            loss: 3,
            gain: 3,
        },
        Case {
            source: "Falkenrath Noble",
            others: two,
            spell: "Wrath of God",
            loss: 3,
            gain: 3,
        },
        Case {
            source: "Vengeful Bloodwitch",
            others: two,
            spell: "Wrath of God",
            loss: 3,
            gain: 3,
        },
        Case {
            source: "Zulaport Cutthroat",
            others: two,
            spell: "Wrath of God",
            loss: 3,
            gain: 3,
        },
        // "Another creature": only the two others (1 damage each).
        Case {
            source: "Vindictive Vampire",
            others: two,
            spell: "Wrath of God",
            loss: 2,
            gain: 2,
        },
        Case {
            source: "Elas il-Kor, Sadistic Pilgrim",
            others: two,
            spell: "Wrath of God",
            loss: 2,
            gain: 0,
        },
        // Only Saprolings: two Saprolings and a Grizzly Bears die with Slimefoot.
        Case {
            source: "Slimefoot, the Stowaway",
            others: &["Saproling", "Saproling", "Grizzly Bears"],
            spell: "Wrath of God",
            loss: 2,
            gain: 2,
        },
        // Itself, a creature and a noncreature artifact (Let the World Burn).
        Case {
            source: "Al Bhed Salvagers",
            others: &["Grizzly Bears", "Mind Stone"],
            spell: "What Must Be Done",
            loss: 3,
            gain: 3,
        },
        Case {
            source: "Susurian Voidborn",
            others: &["Grizzly Bears", "Mind Stone"],
            spell: "What Must Be Done",
            loss: 3,
            gain: 3,
        },
        // A noncreature source leaving with two creatures (the artifact itself doesn't
        // count: it isn't a creature).
        Case {
            source: "Cauldron of Essence",
            others: two,
            spell: "What Must Be Done",
            loss: 2,
            gain: 2,
        },
        Case {
            source: "Bastion of Remembrance",
            others: two,
            spell: "Planar Cleansing",
            loss: 2,
            gain: 2,
        },
        // Two outlaws (Rogues) and a non-outlaw.
        Case {
            source: "Rakish Crew",
            others: &[
                "Bane Alley Blackguard",
                "Bane Alley Blackguard",
                "Grizzly Bears",
            ],
            spell: "Planar Cleansing",
            loss: 2,
            gain: 2,
        },
    ];
    for c in &cases {
        run_case(c);
    }
}

#[test]
fn al_bhed_salvagers_sees_a_noncreature_artifact_die() {
    cr!("700.4");
    ruling!(
        "Al Bhed Salvagers",
        "The term \"dies\" isn't exclusive to creatures. Al Bhed Salvagers's ability will trigger when an artifact is put into a graveyard from the battlefield, even if that artifact wasn't a creature."
    );
    supported("Al Bhed Salvagers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Al Bhed Salvagers");
    let stone = t.battlefield(P0, "Mind Stone");
    kill(&mut t, stone);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn elas_sees_creatures_entering_with_it() {
    cr!("603.6a");
    ruling!(
        "Elas il-Kor, Sadistic Pilgrim",
        "If one or more other creatures enter the battlefield under your control at the same time that Elas il-Kor enters the battlefield, its second ability will trigger for each of those other creatures."
    );
    supported("Elas il-Kor, Sadistic Pilgrim");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Elas il-Kor, Sadistic Pilgrim");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    // Living Death returns all three at once: two triggers (not one for Elas itself).
    cast_spell(&mut t, P0, "Living Death", &[]);
    assert_eq!(
        t.named_on_battlefield("Elas il-Kor, Sadistic Pilgrim")
            .len(),
        1
    );
    assert_eq!(t.life(P0), 22);
}

#[test]
fn kheru_bloodsucker_sees_its_own_last_toughness() {
    cr!("603.10a");
    ruling!(
        "Kheru Bloodsucker",
        "Kheru Bloodsucker’s triggered ability will trigger when it dies if its toughness is 4 or greater."
    );
    supported("Kheru Bloodsucker");
    for (counters, triggers) in [(2u32, true), (1, false)] {
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, "Kheru Bloodsucker");
        put_plus(&mut t, k, counters);
        kill(&mut t, k);
        t.resolve_all();
        let n = if triggers { 2 } else { 0 };
        assert_eq!((t.life(P0), t.life(P1)), (20 + n, 20 - n), "{counters}");
    }
}

#[test]
fn homicide_investigator_triggers_on_its_own_death_unless_its_a_token() {
    cr!("603.10a");
    ruling!(
        "Homicide Investigator",
        "Homicide Investigator's ability triggers when it dies if it's not a token."
    );
    supported("Homicide Investigator");
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    let hi = t.battlefield(P0, "Homicide Investigator");
    cast_spell(&mut t, P0, "Cackling Counterpart", &[Entity::Object(hi)]);
    let copy = tokens(&t, P0)[0];
    assert_eq!(t.obj(copy).chars.name, "Homicide Investigator");
    // The token copy dying: neither ability triggers (it isn't a nontoken creature).
    kill(&mut t, copy);
    t.resolve_all();
    assert_eq!(clues(&t, P0), 0);
    // The nontoken one dying alone: its own ability sees it.
    kill(&mut t, hi);
    t.resolve_all();
    assert_eq!(clues(&t, P0), 1);
}

#[test]
fn homicide_investigator_investigates_once_for_creatures_dying_together() {
    cr!("603.2c", "603.10a");
    interp!(
        "homicide-investigator-triggers-once",
        "ruling-contradicting-oracle-gives-way"
    );
    // Oracle: "Whenever one or more nontoken creatures you control die, investigate. This
    // ability triggers only once each turn." Creatures dying together are one event: one
    // trigger, also when the Investigator dies with them. Its 2024-04-12 ruling "If
    // Homicide Investigator dies at the same time as one or more other nontoken creatures
    // you control, Homicide Investigator's ability triggers for each of them." contradicts
    // both sentences of this text: it is Midnight Reaper's per-creature ruling (Guilds of
    // Ravnica release notes, 2018-09-20) with the name swapped. By the user's decision of
    // Nov 5 (the card's text wins over a ruling that contradicts it) it gives way: exempt
    // in docs/rulings-exemptions/rulings-P005.tsv and logged in
    // docs/RULING_DISCREPANCIES.md for review.
    supported("Homicide Investigator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Homicide Investigator");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    wrath(&mut t, P0);
    assert!(t.named_on_battlefield("Homicide Investigator").is_empty());
    assert_eq!(clues(&t, P0), 1);
    // Only once each turn: a second death that turn doesn't trigger it; the next turn's
    // does.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Homicide Investigator");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let lions = t.battlefield(P0, "Savannah Lions");
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(clues(&t, P0), 1);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(clues(&t, P0), 1);
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    kill(&mut t, lions);
    t.resolve_all();
    assert_eq!(clues(&t, P0), 2);
    // The ruling's own situation after the ability already triggered this turn: the
    // Investigator and two other nontoken creatures die together, and no Clue is made
    // (the ruling would give one for each of them).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Homicide Investigator");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Savannah Lions");
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(clues(&t, P0), 1);
    wrath(&mut t, P0);
    t.resolve_all();
    assert!(t.named_on_battlefield("Homicide Investigator").is_empty());
    assert!(t.named_on_battlefield("Savannah Lions").is_empty());
    assert_eq!(clues(&t, P0), 1);
}

#[test]
fn thieving_amalgam_owner_left_the_game() {
    cr!("603.10a", "800.4a", "704.3");
    ruling!(
        "Thieving Amalgam",
        "If a creature you control but don't own dies at the same time that its owner leaves the game, Thieving Amalgam's last ability triggers. No player loses 2 life, but you gain 2 life."
    );
    supported("Thieving Amalgam");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Thieving Amalgam");
    let m = t.battlefield(P2, "Grizzly Bears");
    crate::r_s06_common::give_control(&mut t, m, P0);
    assert_eq!(t.obj(m).controller, P0);
    // Earthquake for 2 (cast by P1) kills the stolen 2/2 and P2 (at 2 life) at once.
    let p1_before = t.life(P1);
    dies_with_its_controller(&mut t, P2, 2);
    assert!(t.has_lost(P2));
    assert!(!t.on_battlefield(m));
    assert_eq!(t.life(P0), 20 - 2 + 2, "Earthquake's 2, then the gain of 2");
    assert_eq!(t.life(P1), p1_before - 2, "only Earthquake's damage");
}

#[test]
fn controller_loses_before_dies_triggers_go_on_the_stack() {
    cr!("704.3", "704.5a");
    ruling!(
        "Thieving Amalgam",
        "If a creature you control but don't own dies at the same time that your life total becomes 0 or less, you lose the game before Thieving Amalgam's last ability can save you."
    );
    ruling!(
        "Vindictive Vampire",
        "If your life total is brought to 0 or less at the same time that creatures you control are dealt lethal damage, you lose the game before Vindictive Vampire’s triggered ability goes on the stack."
    );
    ruling!(
        "Bastion of Remembrance",
        "If your life total is brought to 0 or less at the same time that creatures you control are dealt lethal damage, you lose the game before the last ability goes on the stack."
    );
    for source in [
        "Thieving Amalgam",
        "Vindictive Vampire",
        "Bastion of Remembrance",
    ] {
        supported(source);
        let mut t = TestGame::new(3);
        t.battlefield(P0, source);
        // A creature P0 controls but P2 owns (for Thieving Amalgam), and one P0 owns.
        let stolen = t.battlefield(P2, "Grizzly Bears");
        crate::r_s06_common::give_control(&mut t, stolen, P0);
        t.battlefield(P0, "Grizzly Bears");
        let p2 = t.life(P2);
        dies_with_its_controller(&mut t, P0, 2);
        assert!(t.has_lost(P0), "{source}");
        assert!(t.g.stack.is_empty(), "{source}");
        assert!(t.in_graveyard(P2, "Grizzly Bears"), "{source}");
        // Earthquake's 2 only: no trigger made P2 (or anyone) lose life.
        assert_eq!(t.life(P2), p2 - 2, "{source}");
    }
}

#[test]
fn two_headed_giant_each_opponent_loses() {
    cr!("810.9");
    ruling!(
        "Bastion of Remembrance",
        "In a Two-Headed Giant game, Bastion of Remembrance's last ability causes the opposing team to lose 2 life and you to gain 1 life."
    );
    ruling!(
        "Vindictive Vampire",
        "In a Two-Headed Giant game, Vindictive Vampire’s ability causes the opposing team to lose 2 life and you gain 1 life."
    );
    ruling!(
        "Elas il-Kor, Sadistic Pilgrim",
        "In a Two-Headed Giant game, Elas il-Kor’s last ability causes the opposing team to lose 2 life."
    );
    for (source, gain) in [
        ("Bastion of Remembrance", 1),
        ("Vindictive Vampire", 1),
        ("Elas il-Kor, Sadistic Pilgrim", 0),
    ] {
        supported(source);
        let mut t = two_headed_giant();
        t.battlefield(P0, source);
        let bears = t.battlefield(P0, "Grizzly Bears");
        kill(&mut t, bears);
        t.resolve_all();
        assert_eq!(t.life(P2), 28, "{source}");
        assert_eq!(t.life(P3), 28, "{source}");
        assert_eq!(t.life(P0), 30 + gain, "{source}");
    }
}

/// Puts a Monster Role token (an Aura P0 controls) on `id` with Monstrous Rage.
fn monster_role(t: &mut TestGame, id: ObjectId) -> ObjectId {
    cast_spell(t, P0, "Monstrous Rage", &[Entity::Object(id)]);
    crate::r_s15_common::roles_on(t, id)[0]
}

#[test]
fn wicked_visitor_sees_role_tokens_go_to_the_graveyard() {
    cr!("111.7", "704.5m");
    ruling!(
        "Wicked Visitor",
        "Enchantment tokens (such as Roles) that are sacrificed, destroyed, or would otherwise go to the graveyard are put into their owner's graveyard before ceasing to exist. If you controlled the token, Wicked Visitor's ability will trigger."
    );
    supported("Wicked Visitor");
    supported("Monstrous Rage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wicked Visitor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Destroyed.
    let role = monster_role(&mut t, bears);
    kill(&mut t, role);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Put into the graveyard by a state-based action (its creature died).
    monster_role(&mut t, bears);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn wicked_visitor_misses_auras_that_go_after_it_left() {
    cr!("603.10a", "704.5m");
    ruling!(
        "Wicked Visitor",
        "If Wicked Visitor leaves the battlefield while it has an Aura you control attached to it or at the same time as another creature with an Aura you control attached to it, its ability won't trigger for those enchantments unless they were destroyed by the same effect that caused Wicked Visitor to leave the battlefield."
    );
    supported("Wicked Visitor");
    // The Aura on Wicked Visitor itself, Visitor destroyed alone: the Aura goes later,
    // as a state-based action.
    let mut t = TestGame::new(2);
    let wv = t.battlefield(P0, "Wicked Visitor");
    monster_role(&mut t, wv);
    kill(&mut t, wv);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // The Aura on another creature dying with Visitor (Wrath of God).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wicked Visitor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    monster_role(&mut t, bears);
    wrath(&mut t, P0);
    assert_eq!(t.life(P1), 20);
    // Both Auras destroyed by the same effect as Visitor (Planar Cleansing).
    let mut t = TestGame::new(2);
    let wv = t.battlefield(P0, "Wicked Visitor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    monster_role(&mut t, wv);
    monster_role(&mut t, bears);
    cleansing(&mut t, P0);
    assert_eq!(t.life(P1), 18);
}
