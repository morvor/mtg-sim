//! Rulings batch P003 — artifacts and legendary permanents entering and leaving: Gandalf
//! the White's additional triggers (CR 603.2d: an ability that triggers an additional
//! time; look-back for leaves-the-battlefield abilities, CR 603.10a), "as enters"
//! replacement effects (CR 614.12), "an artifact entered the battlefield under your
//! control this turn" (Sentinel Sarah Lyons, Akal Pakal), and Reckless Fireweaver in
//! Two-Headed Giant (CR 810.7).

use crate::r_p003_common::*;
use crate::r_s01_common::supported;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn gandalf(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Gandalf the White");
    t.battlefield(p, "Gandalf the White")
}

fn yes_nos_since(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count()
}

#[test]
fn gandalf_the_white_triggers_twice_with_separate_choices() {
    cr!("603.2d", "603.3d", "601.2d");
    ruling!(
        "Gandalf the White",
        "Gandalf the White's effect doesn't copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    supported("Gandalf, Spark Starter");
    supported("Glint Hawk Idol");
    let mut t = TestGame::new(2);
    gandalf(&mut t, P0);
    let bear = t.battlefield(P1, "Grizzly Bears");
    // Gandalf, Spark Starter (legendary): "When Gandalf enters, he deals 3 damage divided
    // as you choose among one, two, or three targets." Two instances: 3 to P1; 2 to the
    // Bears and 1 to P1.
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(bear), pl(P1)]);
    divide(&mut t, P0, &[2, 1]);
    enter(&mut t, P0, "Gandalf, Spark Starter");
    assert_eq!(t.stack_len(), 2, "two instances of the ability");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(bear));
    // Glint Hawk Idol: "Whenever another artifact you control enters, you may have this
    // artifact become a 2/2 Bird artifact creature with flying until end of turn." Each
    // instance asks: no, then yes.
    let idol = t.battlefield(P0, "Glint Hawk Idol");
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(yes_nos_since(&t, from), 2);
    assert!(is_creature(&t, idol));
}

#[test]
fn gandalf_the_white_doesnt_double_as_enters_choices_or_replacement_effects() {
    cr!("603.2d", "614.12", "614.1c");
    ruling!(
        "Gandalf the White",
        "Abilities that apply \"as [this permanent] enters the battlefield,\" such as choosing a color with Gauntlet of Power, are also unaffected."
    );
    ruling!(
        "Gandalf the White",
        "Replacement effects are unaffected by Gandalf the White's ability. For example, a legendary creature that enters the battlefield with one +1/+1 counter on it won't receive an additional +1/+1 counter."
    );
    supported("Gauntlet of Power");
    let mut t = TestGame::new(2);
    gandalf(&mut t, P0);
    let from = t.asked().len();
    let gauntlet = enter(&mut t, P0, "Gauntlet of Power");
    t.resolve_all();
    let color_choices = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("color")))
        .count();
    assert_eq!(color_choices, 1);
    assert!(t.on_battlefield(gauntlet));
    // Verazol (legendary) cast with X = 1: "enters with a +1/+1 counter on it for each
    // mana spent to cast it" — three counters, not six.
    let v = t.hand(P0, "Verazol, the Split Current");
    mana(&mut t, P0, "1GU");
    t.cast(P0, v).x(1).go();
    t.resolve_all();
    let v = t.named_on_battlefield("Verazol, the Split Current")[0];
    assert_eq!(t.counters(v, counters::PLUS1), 3);
}

#[test]
fn gandalf_the_white_doubles_dies_triggers_of_legends_and_artifacts_leaving_with_it() {
    cr!("603.2d", "603.10a", "603.6c");
    ruling!(
        "Gandalf the White",
        "If a legendary permanent or an artifact entering or leaving the battlefield at the same time as Gandalf the White (including Gandalf the White itself) causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
    );
    ruling!(
        "Gandalf the White",
        "The trigger event doesn't have to specifically refer to legendary permanents or to artifacts, nor does it have to specifically refer to when something \"leaves the battlefield.\""
    );
    ruling!(
        "Gandalf the White",
        "You don't need to control the permanent entering or leaving the battlefield, only the permanent that has the triggered ability."
    );
    supported("Blood Artist");
    supported("Wrath of God");
    // Blood Artist: "Whenever this creature or another creature dies, target player loses 1
    // life and you gain 1 life." Wrath of God destroys Gandalf (legendary), Blood Artist,
    // P0's Grizzly Bears and P1's Ornithopter (an artifact): 2 + 1 + 1 + 2 triggers.
    let mut t = TestGame::new(2);
    gandalf(&mut t, P0);
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Ornithopter");
    for _ in 0..6 {
        t.answer_targets(P0, &[pl(P1)]);
    }
    cast_resolve(&mut t, P0, "Wrath of God", "2WW", &[]);
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn gandalf_the_white_sees_types_from_continuous_effects() {
    cr!("603.2d", "603.10a", "613.1d");
    ruling!(
        "Gandalf the White",
        "Look at each legendary permanent and each artifact as it exists on the battlefield, taking into account continuous effects, to determine whether any triggered abilities will trigger multiple times."
    );
    supported("Argent Mutation");
    // Grizzly Bears made an artifact by Argent Mutation ("Target permanent becomes an
    // artifact in addition to its other types until end of turn. Draw a card.") dies:
    // Blood Artist triggers twice. The other Bears: once.
    let mut t = TestGame::new(2);
    gandalf(&mut t, P0);
    t.battlefield(P0, "Blood Artist");
    let art = t.battlefield(P1, "Grizzly Bears");
    let plain = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Argent Mutation", "2U", &[obj(art)]);
    assert!(is_type(&t, art, CardType::Artifact));
    for _ in 0..3 {
        t.answer_targets(P0, &[pl(P1)]);
    }
    kill(&mut t, art);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    kill(&mut t, plain);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn two_gandalfs_trigger_three_times() {
    cr!("603.2d", "707.9b");
    ruling!(
        "Gandalf the White",
        "If you somehow control two copies of Gandalf the White, a legendary permanent or an artifact entering or leaving the battlefield causes abilities to trigger three times, not four."
    );
    supported("Spark Double");
    // Spark Double copying Gandalf isn't legendary, so both stay. Reckless Fireweaver:
    // "Whenever an artifact you control enters, this creature deals 1 damage to each
    // opponent."
    let mut t = TestGame::new(2);
    let g = gandalf(&mut t, P0);
    t.answer_choose(P0, &[obj(g)]);
    cast_resolve(&mut t, P0, "Spark Double", "3U", &[]);
    assert_eq!(t.named_on_battlefield("Gandalf the White").len(), 2);
    add(&mut t, P0, "Reckless Fireweaver");
    enter(&mut t, P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn reckless_fireweaver_deals_two_damage_to_the_opposing_two_headed_giant_team() {
    cr!("810.9");
    ruling!(
        "Reckless Fireweaver",
        "In a Two-Headed Giant game, Reckless Fireweaver's ability causes 2 damage total to be dealt to the opposing team."
    );
    let mut t = crate::r_p050_common::two_headed_giant();
    add(&mut t, P0, "Reckless Fireweaver");
    let life = t.life(P2);
    enter(&mut t, P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.life(P2), life - 2);
    assert_eq!(t.life(P3), life - 2);
}

#[test]
fn sentinel_sarah_lyons_cares_whether_an_artifact_entered_this_turn() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Sentinel Sarah Lyons",
        "Once an artifact enters the battlefield under your control in a given turn, it doesn’t matter what happens to it later."
    );
    ruling!(
        "Sentinel Sarah Lyons",
        "Sentinel Sarah Lyons doesn’t have to have been on the battlefield at the point in the turn where an artifact entered the battlefield under your control."
    );
    ruling!(
        "Sentinel Sarah Lyons",
        "Sentinel Sarah Lyons’s second ability only cares if one or more artifacts entered the battlefield under your control this turn. Your creatures won’t get more powerful if more than one artifact entered the battlefield under your control this turn."
    );
    supported("Sentinel Sarah Lyons");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    // P1's artifact entering doesn't count.
    add(&mut t, P0, "Sentinel Sarah Lyons");
    enter(&mut t, P1, "Ornithopter");
    assert_eq!(t.pt(bear), (2, 2));
    // An Ornithopter enters before Sarah does (start over), then dies; a second one.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let thopter = enter(&mut t, P0, "Ornithopter");
    kill(&mut t, thopter);
    let sarah = enter(&mut t, P0, "Sentinel Sarah Lyons");
    assert_eq!(t.pt(bear), (4, 4));
    assert_eq!(t.pt(sarah), (6, 6));
    enter(&mut t, P0, "Ornithopter");
    assert_eq!(t.pt(bear), (4, 4));
    // Next turn: no longer.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn akal_pakal_triggers_whatever_happened_to_the_artifact() {
    cr!("603.4");
    ruling!(
        "Akal Pakal, First Among Equals",
        "The ability will trigger no matter what happened to the artifact after it entered. It may have left the battlefield, come under another player's control, or stopped being an artifact."
    );
    supported("Akal Pakal, First Among Equals");
    // "At the beginning of each player's end step, if an artifact entered the battlefield
    // under your control this turn, look at the top two cards of your library. Put one of
    // them into your hand and the other into your graveyard."
    for entered in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Akal Pakal, First Among Equals");
        if entered {
            let thopter = enter(&mut t, P0, "Ornithopter");
            kill(&mut t, thopter);
        }
        let (hand, gy) = (t.hand_size(P0), t.graveyard_size(P0));
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let n = entered as usize;
        assert_eq!(t.hand_size(P0), hand + n, "entered: {entered}");
        assert_eq!(t.graveyard_size(P0), gy + n, "entered: {entered}");
    }
}
