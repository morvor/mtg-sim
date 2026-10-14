//! Rulings batch P219 — modular (CR 702.43), monstrosity (CR 701.37), morbid (an ability
//! word, CR 207.2c) and morph (CR 702.37).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, can_cast, destroy};
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s05_common::enter;
use crate::r_s06_common::activate_containing;
use crate::r_s11_common::{spells_copied, triggered_from, turn_face_up};
use crate::r_s13_common::add;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BLASTER: &str = "Blaster, Combat DJ // Blaster, Morale Booster";

#[test]
fn granted_modular_triggers_when_blaster_dies_at_the_same_time() {
    cr!("702.43a", "603.10a");
    ruling!(
        "Blaster, Combat DJ // Blaster, Morale Booster",
        "If Blaster, Combat DJ dies at the same time as one or more artifact creatures or Vehicles, the modular abilities it was granting to each of them will trigger."
    );
    supported(BLASTER);
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    t.battlefield(P0, BLASTER);
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    let thopter = enter(&mut t, P0, "Ornithopter");
    assert_eq!(t.counters(thopter, "+1/+1"), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(colossus)]);
    let wrath = in_hand_with_mana(&mut t, P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(t.counters(colossus, "+1/+1"), 1);
}

#[test]
fn entering_with_blaster_gives_no_modular_counter() {
    cr!("702.43a", "614.12");
    ruling!(
        "Blaster, Combat DJ // Blaster, Morale Booster",
        "If Blaster, Combat DJ somehow enters the battlefield at the same time as one or more other artifact creatures or Vehicles, none of those creatures or Vehicles will have modular 1 due to Blaster's ability until after they have entered, so they will not enter with the additional +1/+1 counter."
    );
    supported("Living Death");
    let mut t = TestGame::new(2);
    t.graveyard(P0, BLASTER);
    let thopter = t.graveyard(P0, "Ornithopter");
    let death = in_hand_with_mana(&mut t, P0, "Living Death");
    t.cast(P0, death).go();
    t.resolve_all();
    let thopter = t.g.current(thopter);
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.named_on_battlefield("Blaster, Combat DJ").len(), 1);
    assert_eq!(t.counters(thopter, "+1/+1"), 0);
}

#[test]
fn modular_counts_the_counters_it_died_with() {
    cr!("702.43a", "704.5f", "704.5q", "603.10a");
    ruling!(
        "Power Depot",
        "If Power Depot is a creature and gets enough -1/-1 counters put on it to cause it to go to the graveyard, modular will put a number of +1/+1 counters on the target artifact creature equal to the number of +1/+1 counters on this creature before it left the battlefield."
    );
    ruling!(
        "Scrapyard Recombiner",
        "If a creature with modular receives enough -1/-1 counters to cause it to be destroyed by lethal damage or put into its owner’s graveyard for having 0 or less toughness, modular will put a number of +1/+1 counters on the target artifact creature equal to the number of +1/+1 counters on this creature before it left the battlefield. That’s because modular checks the creature as it last existed on the battlefield, and it still had +1/+1 counters on it at that point."
    );
    ruling!(
        "Blaster, Combat DJ // Blaster, Morale Booster",
        "If a creature with one or more +1/+1 counters on it gets enough -1/-1 counters put on it to cause it to die, it leaves the battlefield with all those counters on it. The modular ability will put a number of +1/+1 counters on the target artifact creature equal to the number of +1/+1 counters on this creature before it left the battlefield."
    );
    supported("Power Depot");
    supported("Scrapyard Recombiner");
    supported("March of the Machines");
    // (card, +1/+1 counters it has, -1/-1 counters that kill it)
    for (name, plus, minus) in [
        ("Power Depot", 1, 1),
        ("Scrapyard Recombiner", 2, 2),
        ("Ornithopter", 1, 3),
    ] {
        let mut t = TestGame::new(2);
        // March of the Machines: "Each noncreature artifact is an artifact creature with
        // power and toughness each equal to its mana value." (Power Depot: 0/0.)
        t.battlefield(P0, "March of the Machines");
        // Blaster gives the Ornithopter modular 1 (and would give the others another).
        if name == "Ornithopter" {
            t.battlefield(P0, BLASTER);
        }
        let colossus = t.battlefield(P0, "Darksteel Colossus");
        let c = enter(&mut t, P0, name);
        t.resolve_all();
        assert_eq!(t.counters(c, "+1/+1"), plus, "{name}");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Object(colossus)]);
        add(&mut t, c, "-1/-1", minus);
        t.settle();
        t.resolve_all();
        assert!(!t.on_battlefield(c), "{name}");
        assert_eq!(t.counters(colossus, "+1/+1"), plus, "{name}");
    }
}

#[test]
fn nemesis_of_mortals_cant_reduce_colored_mana() {
    cr!("601.2f", "602.2b", "118.7d");
    ruling!(
        "Nemesis of Mortals",
        "Neither the cost-reducing ability of Nemesis of Mortals nor the condition of its monstrosity ability can reduce the colored mana requirement of those costs."
    );
    supported("Nemesis of Mortals");
    for (forests, ok) in [(1, false), (2, true)] {
        // Ten creature cards in the graveyard: {4}{G}{G} becomes {G}{G}; {7}{G}{G} too.
        let mut t = TestGame::new(2);
        for _ in 0..10 {
            t.graveyard(P0, "Grizzly Bears");
        }
        t.lands(P0, "Forest", forests);
        let nemesis = t.hand(P0, "Nemesis of Mortals");
        assert_eq!(can_cast(&mut t, P0, nemesis, CastMethod::Normal), ok);
        let on = t.battlefield(P0, "Nemesis of Mortals");
        assert_eq!(can_activate(&mut t, P0, on), ok);
        if ok {
            activate_containing(&mut t, P0, on, "Monstrosity").expect("activated");
            t.resolve_all();
            assert_eq!(t.counters(on, "+1/+1"), 5);
        }
    }
}

#[test]
fn sinuous_vermin_gaining_menace_after_blocks_doesnt_undo_the_block() {
    cr!("509.1h", "702.111b", "506.4");
    ruling!(
        "Sinuous Vermin",
        "Once Sinuous Vermin has been legally blocked by one creature, activating the monstrosity ability to give it menace won’t change or undo that block."
    );
    supported("Sinuous Vermin");
    let mut t = TestGame::new(2);
    let vermin = t.battlefield(P0, "Sinuous Vermin");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 3);
    crate::r_s03_common::to_blockers(&mut t, &[(vermin, Entity::Player(P1))], &[(wall, vermin)]);
    activate_containing(&mut t, P0, vermin, "Monstrosity").expect("activated");
    t.resolve_all();
    assert!(t
        .obj_now(vermin)
        .has_keyword(mtg_engine::keywords::KeywordKind::Menace));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(wall).damage, 5);
}

#[test]
fn hydra_broodmaster_x_is_the_monstrosity_x() {
    cr!("701.37c", "107.3m");
    ruling!(
        "Hydra Broodmaster",
        "The value of each X in Hydra Broodmaster's last ability is equal to the value chosen for X when its monstrosity ability was activated."
    );
    supported("Hydra Broodmaster");
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Hydra Broodmaster");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 6);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    activate_containing(&mut t, P0, hydra, "Monstrosity").expect("activated");
    t.resolve_all();
    assert_eq!(t.counters(hydra, "+1/+1"), 3);
    let tokens = tokens(&t, P0);
    assert_eq!(tokens.len(), 3);
    assert!(tokens.iter().all(|h| t.pt(*h) == (3, 3)));
}

/// Kills a Grizzly Bears of `p` so that a creature died this turn.
fn a_creature_died(t: &mut TestGame, p: PlayerId) {
    let bears = t.battlefield(p, "Grizzly Bears");
    destroy(t, bears);
}

#[test]
fn morkrut_banshee_must_target_itself_and_triggers_once() {
    cr!("603.3d", "603.4", "207.2c");
    ruling!(
        "Morkrut Banshee",
        "If there are no other creatures on the battlefield when the morbid ability triggers, the ability must target Morkrut Banshee itself."
    );
    ruling!(
        "Morkrut Banshee",
        "Morkrut Banshee's morbid ability triggers only once, not once for each creature that has died this turn. If no creatures have died by the time it enters the battlefield, its ability won't trigger at all."
    );
    supported("Morkrut Banshee");
    // Two creatures died; it's the only creature: it targets itself (P0 names no target).
    let mut t = TestGame::new(2);
    a_creature_died(&mut t, P0);
    a_creature_died(&mut t, P1);
    t.answer_targets(P0, &[]);
    let banshee = enter(&mut t, P0, "Morkrut Banshee");
    assert_eq!(triggered_from(&t, banshee), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Morkrut Banshee"));
    // Nothing died: no trigger.
    let mut t = TestGame::new(2);
    let banshee = enter(&mut t, P0, "Morkrut Banshee");
    assert_eq!(triggered_from(&t, banshee), 0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn malicious_affliction_copy_isnt_cast() {
    cr!("707.10", "601.2i", "603.4");
    ruling!(
        "Malicious Affliction",
        "The copy of Malicious Affliction is created on the stack. It’s not cast, so the copy won’t cause the morbid ability to trigger again."
    );
    supported("Malicious Affliction");
    let mut t = TestGame::new(2);
    a_creature_died(&mut t, P1);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Gray Ogre");
    let spell = in_hand_with_mana(&mut t, P0, "Malicious Affliction");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    let spell = t.cast(P0, spell).target(Entity::Object(a)).go();
    t.resolve_all();
    assert_eq!(spells_copied(&t), 1);
    assert_eq!(triggered_from(&t, spell), 1);
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn reaper_from_the_abyss_must_target_your_own_creature() {
    cr!("603.3d", "115.1");
    ruling!(
        "Reaper from the Abyss",
        "The morbid ability is mandatory. If you control the only non-Demon creature when the ability triggers, you must choose it as the target."
    );
    supported("Reaper from the Abyss");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reaper from the Abyss");
    let ogre = t.battlefield(P0, "Gray Ogre");
    a_creature_died(&mut t, P1);
    t.answer_targets(P0, &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(ogre));
    assert_eq!(t.named_on_battlefield("Reaper from the Abyss").len(), 1);
}

#[test]
fn woodland_sleuth_can_return_itself() {
    cr!("608.2c", "603.4");
    ruling!(
        "Woodland Sleuth",
        "Woodland Sleuth could die in response to its own morbid ability. If this happens, the ability could return Woodland Sleuth to its owner’s hand."
    );
    supported("Woodland Sleuth");
    let mut t = TestGame::new(2);
    // A token died (it isn't a card in a graveyard afterward).
    let token = crate::r_s02_common::create_token(&mut t, P0, "Grizzly Bears");
    destroy(&mut t, token);
    let sleuth = enter(&mut t, P0, "Woodland Sleuth");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, sleuth);
    t.resolve_all();
    assert!(t.in_hand(P0, "Woodland Sleuth"));
}

#[test]
fn rattleclaw_mystics_face_up_mana_comes_from_a_triggered_ability() {
    cr!("702.37e", "116.2b", "605.1b");
    ruling!(
        "Rattleclaw Mystic",
        "Although turning a face-down creature with morph face up doesn't use the stack, Rattleclaw Mystic's triggered ability does use the stack, and players can respond to that ability. Notably, you can't turn Rattleclaw Mystic face up while casting a spell or activating an ability. You'll need to turn Rattleclaw Mystic face up and have its triggered ability resolve before you can spend that mana."
    );
    supported("Rattleclaw Mystic");
    let mut t = TestGame::new(2);
    let mystic = crate::r_s12_common::morph(&mut t, P0, "Rattleclaw Mystic");
    t.lands(P0, "Wastes", 2);
    assert!(turn_face_up(&mut t, P0, mystic));
    t.settle();
    // The mana ability is a trigger on the stack: no mana yet.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve();
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(
        (
            pool.count(ManaType::G),
            pool.count(ManaType::U),
            pool.count(ManaType::R)
        ),
        (1, 1, 1)
    );
}
