//! Rulings batch P003 — anthems and bonuses from Auras and Equipment: they apply as soon
//! as their source is on the battlefield (CR 611.3a, 614.12), damage stays marked when
//! they stop (CR 120.6, 704.5g), conditions about the turn switch at once (CR 611.3a),
//! "if you attacked with two or more creatures this turn" looks back (CR 508.1), and a
//! token's counters put after it enters (CR 603.6a, 111.1).

use crate::r_p003_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::attach_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn archangel_of_strife_enters_with_the_bonus_already_applied() {
    cr!("614.12", "611.3a", "613.4c");
    ruling!(
        "Archangel of Strife",
        "Archangel of Strife's bonuses apply immediately. Depending on your choice, Archangel of Strife will enter as either a 9/6 or 6/9 creature."
    );
    supported("Archangel of Strife");
    for (choice, pt) in [(0usize, (9, 6)), (1, (6, 9))] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.answer(P1, DecisionKind::Option, Answer::Index(1 - choice));
        // The choices are made as it enters (a replacement effect, not a trigger to wait
        // for): it's on the battlefield with its bonus, nothing on the stack.
        let a = enter(&mut t, P0, "Archangel of Strife");
        assert_eq!(t.stack_len(), 0);
        assert_eq!(t.pt(a), pt, "war: {}", choice == 0);
        // P1 chose the other: their Bears get the other bonus.
        let bear_pt = if choice == 0 { (2, 5) } else { (5, 2) };
        assert_eq!(t.pt(bear), bear_pt);
    }
}

#[test]
fn damage_stays_marked_when_an_anthem_leaves() {
    cr!("120.6", "704.5g", "613.4c");
    ruling!(
        "Benalish Marshal",
        "Because damage remains marked on a creature until it’s removed as the turn ends, nonlethal damage dealt to a creature you control may become lethal if Benalish Marshal leaves the battlefield during that turn."
    );
    ruling!(
        "Chief of the Foundry",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to an artifact creature you control may become lethal if Chief of the Foundry leaves the battlefield during that turn."
    );
    ruling!(
        "Tempered Steel",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to artifact creatures you control may become lethal if Tempered Steel leaves the battlefield during that turn."
    );
    // The anthem, the creature it pumps, its toughness with the bonus.
    for (anthem, creature, tough) in [
        ("Benalish Marshal", "Grizzly Bears", 3),
        ("Chief of the Foundry", "Ornithopter", 3),
        ("Tempered Steel", "Ornithopter", 4),
    ] {
        supported(anthem);
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, anthem);
        let c = t.battlefield(P0, creature);
        assert_eq!(t.pt(c).1, tough, "{anthem}");
        damage(&mut t, a, c, tough as u32 - 1);
        assert!(t.on_battlefield(c), "{anthem}: not lethal yet");
        kill(&mut t, a);
        assert!(!t.on_battlefield(c), "{anthem}: lethal now");
    }
    // Champion of the Flame: "+2/+2 for each Aura and Equipment attached to it", 1/1;
    // with Leonin Scimitar (+1/+1): 4/4. 3 damage, then the Scimitar moves to a Bears.
    ruling!(
        "Champion of the Flame",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Champion of the Flame may become lethal if an Aura or Equipment attached to it leaves the battlefield or becomes attached to another creature during that turn."
    );
    supported("Champion of the Flame");
    supported("Leonin Scimitar");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Champion of the Flame");
    let sword = attach_new(&mut t, P0, "Leonin Scimitar", obj(champ));
    assert_eq!(t.pt(champ), (4, 4));
    damage(&mut t, sword, champ, 3);
    assert!(t.on_battlefield(champ));
    let bear = t.battlefield(P0, "Grizzly Bears");
    activate_resolve(&mut t, P0, sword, "Equip", "1", &[obj(bear)]);
    assert!(!t.on_battlefield(champ));
}

#[test]
fn glory_of_warfare_switches_bonuses_with_the_turn() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Glory of Warfare",
        "There is never a point at which Glory of Warfare will give a creature you control +2/+2 or +0/+0."
    );
    supported("Glory of Warfare");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glory of Warfare");
    // Savannah Lions (2/1).
    let lions = t.battlefield(P0, "Savannah Lions");
    assert_eq!(t.pt(lions), (4, 1));
    t.advance_to(P0, Step::End);
    assert_eq!(t.pt(lions), (4, 1));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(lions), (2, 3));
    t.advance_to(P1, Step::End);
    assert_eq!(t.pt(lions), (2, 3));
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.pt(lions), (4, 1));
}

#[test]
fn wedding_announcement_looks_back_at_this_turns_attacks() {
    cr!("508.1");
    ruling!(
        "Wedding Announcement // Wedding Festivity",
        "Wedding Announcement's ability looks back in time to the combat phase of this turn, even if it wasn't yet on the battlefield when you attacked. If you attacked with at least two creatures this turn, you will draw a card as the ability resolves. This is true even if one or more of those permanents is no longer a creature or no longer on the battlefield."
    );
    supported("Wedding Announcement // Wedding Festivity");
    for attackers in [1usize, 2] {
        let mut t = TestGame::new(2);
        let bears: Vec<ObjectId> = (0..attackers)
            .map(|_| t.battlefield(P0, "Grizzly Bears"))
            .collect();
        let atk: Vec<(ObjectId, Entity)> = bears.iter().map(|b| (*b, pl(P1))).collect();
        t.attack(&atk, &[]);
        for b in &bears {
            kill(&mut t, *b);
        }
        t.advance_to(P0, Step::PostcombatMain);
        add(&mut t, P0, "Wedding Announcement // Wedding Festivity");
        let hand = t.hand_size(P0);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let humans = crate::r_s01_common::with_subtype(&t, P0, "Human").len();
        if attackers == 2 {
            assert_eq!(t.hand_size(P0), hand + 1);
            assert_eq!(humans, 0);
        } else {
            assert_eq!(t.hand_size(P0), hand);
            assert_eq!(humans, 1);
        }
    }
}

#[test]
fn equipment_bonuses_add_up_and_golem_skin_gauntlets_counts_itself() {
    cr!("613.4c", "301.5");
    ruling!(
        "Goblin Gaveleer",
        "Goblin Gaveleer's bonus is in addition to whatever bonus the Equipment gives it."
    );
    ruling!(
        "Golem-Skin Gauntlets",
        "Golem-Skin Gauntlets is counted by its own ability, so it gives at least +1/+0."
    );
    supported("Goblin Gaveleer");
    supported("Golem-Skin Gauntlets");
    // Goblin Gaveleer (1/1, "+2/+0 for each Equipment attached to it") with Leonin
    // Scimitar (+1/+1): 4/2.
    let mut t = TestGame::new(2);
    let gav = t.battlefield(P0, "Goblin Gaveleer");
    attach_new(&mut t, P0, "Leonin Scimitar", obj(gav));
    assert_eq!(t.pt(gav), (4, 2));
    // Golem-Skin Gauntlets ("Equipped creature gets +1/+0 for each Equipment attached to
    // it") alone: +1/+0; with the Scimitar too: +2/+0 and +1/+1.
    let bear = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Golem-Skin Gauntlets", obj(bear));
    assert_eq!(t.pt(bear), (3, 2));
    attach_new(&mut t, P0, "Leonin Scimitar", obj(bear));
    assert_eq!(t.pt(bear), (5, 3));
}

#[test]
fn animal_friend_squirrel_enters_as_a_1_1_before_its_counters() {
    cr!("603.6a", "603.2", "608.2c");
    ruling!(
        "Animal Friend",
        "The token will enter the battlefield and then have counters put on it. In most cases, the game will see a 1/1 creature entering the battlefield for the purposes of any abilities that may trigger"
    );
    supported("Animal Friend");
    // A Grizzly Bears with Animal Friend and two Leonin Scimitars attacks: the Squirrel
    // gets two +1/+1 counters, after it entered as a 1/1. Garruk's Packleader ("Whenever
    // another creature you control with power 3 or greater enters, you may draw a card")
    // doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Packleader");
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Animal Friend", "1G", &[obj(bear)]);
    attach_new(&mut t, P0, "Leonin Scimitar", obj(bear));
    attach_new(&mut t, P0, "Leonin Scimitar", obj(bear));
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.attack(&[(bear, pl(P1))], &[]);
    let squirrels = crate::r_s01_common::with_subtype(&t, P0, "Squirrel");
    assert_eq!(squirrels.len(), 1);
    assert_eq!(t.pt(squirrels[0]), (3, 3));
    assert_eq!(t.hand_size(P0), hand);
}
