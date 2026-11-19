//! Rulings batch P088 — "[types] entering (or dying) don't cause abilities to trigger"
//! (Hushbringer, Hushwing Gryff, Tocatli Honor Guard, Doorkeeper Thrull; CR 603.2,
//! 603.6a, 603.6c, 603.10a) and Containment Priest's replacement effect (CR 614.12).

use crate::r_p088_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts Diamond Knight onto the battlefield for P0, choosing red.
fn red_diamond_knight(t: &mut TestGame) -> ObjectId {
    let i = Color::ALL.iter().position(|c| *c == Color::Red).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let k = t.enter(P0, "Diamond Knight");
    t.settle();
    k
}

#[test]
fn as_enters_choices_still_happen_under_hushbringer_and_its_kin() {
    cr!("603.6d", "614.12", "614.1c");
    ruling!(
        "Hushbringer",
        "Abilities that apply \"as [this creature] enters the battlefield,\" such as choosing a color with Diamond Knight, are unaffected."
    );
    ruling!(
        "Hushwing Gryff",
        "Abilities that apply “as [this creature] enters the battlefield,” such as choosing a creature to copy with Mercurial Pretender, are also unaffected."
    );
    supported("Hushbringer");
    supported("Diamond Knight");
    supported("Mercurial Pretender");
    // Diamond Knight under Hushbringer: red is chosen as it enters, and casting a red
    // spell (not an enters trigger) gives it a counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushbringer");
    let k = red_diamond_knight(&mut t);
    assert_eq!(t.stack_len(), 0);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.counters(k, "+1/+1"), 1);
    // Mercurial Pretender under Hushwing Gryff enters as a copy of Elvish Visionary, whose
    // enters ability ("When this creature enters, draw a card.") doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushwing Gryff");
    let visionary = t.battlefield(P0, "Elvish Visionary");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(visionary)]);
    let m = t.enter(P0, "Mercurial Pretender");
    t.settle();
    assert_eq!(t.obj_now(m).chars.name.as_str(), "Elvish Visionary");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn hushbringer_and_put_into_a_graveyard_from_anywhere() {
    cr!("603.2", "603.10a", "700.4");
    ruling!(
        "Hushbringer",
        "An ability that triggers when a card is put into a graveyard \"from anywhere\" may still trigger even if a creature dies. It will trigger if the ability is of a dying creature that triggers only when that creature is put into a graveyard from anywhere, if Hushbringer is the card being put into a graveyard, or if Hushbringer leaves the battlefield at the same time as the dying creature."
    );
    supported("Profane Memento");
    supported("Dread");
    // Profane Memento: "Whenever a creature card is put into an opponent's graveyard from
    // anywhere, you gain 1 life."
    // A creature dying while Hushbringer stays: it doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Profane Memento");
    t.battlefield(P0, "Hushbringer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Hushbringer itself being put into a graveyard: it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Profane Memento");
    let hb = t.battlefield(P1, "Hushbringer");
    destroy(&mut t, hb);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Hushbringer leaving at the same time as the dying creature: both trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Profane Memento");
    t.battlefield(P1, "Hushbringer");
    t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // The dying creature's own "When Dread is put into a graveyard from anywhere, shuffle
    // it into its owner's library." triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushbringer");
    let dread = t.battlefield(P1, "Dread");
    destroy(&mut t, dread);
    assert_eq!(triggers_on_stack(&t, "from anywhere"), 1);
    t.resolve_all();
    assert_eq!(t.zone(dread), Zone::Library(P1));
}

#[test]
fn hushbringer_stops_leaves_the_battlefield_abilities_of_dying_creatures() {
    cr!("603.6c", "603.10a", "700.4");
    ruling!(
        "Hushbringer",
        "An ability that triggers when a creature \"leaves the battlefield\" won't trigger if that creature leaves the battlefield by dying."
    );
    supported("Circuit Mender");
    // Circuit Mender: "When this creature leaves the battlefield, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushbringer");
    let m = t.battlefield(P0, "Circuit Mender");
    let hand = t.hand_size(P0);
    destroy(&mut t, m);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Leaving another way (exiled): it triggers.
    let m = t.battlefield(P0, "Circuit Mender");
    t.g.exile_object(m, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn hushbringer_judges_the_permanent_as_it_existed_on_the_battlefield() {
    cr!("603.10a", "613.1d", "700.4");
    ruling!(
        "Hushbringer",
        "Look at each permanent as it exists on the battlefield, taking into account continuous effects that apply to it after entering or before dying, to determine whether any triggered abilities will be stopped from triggering. For example, if a land that has become a creature dies, an ability that triggers when it dies doesn't trigger."
    );
    supported("Mutavault");
    supported("Blood Artist");
    // Mutavault: "{1}: Until end of turn, this land becomes a 2/2 creature with all
    // creature types." Blood Artist: "Whenever this creature or another creature dies,
    // target player loses 1 life and you gain 1 life."
    for hush in [false, true] {
        let mut t = TestGame::new(2);
        if hush {
            t.battlefield(P0, "Hushbringer");
        }
        t.battlefield(P0, "Blood Artist");
        let vault = t.battlefield(P0, "Mutavault");
        colorless(&mut t, P0, 1);
        activate_containing(&mut t, P0, vault, "becomes").expect("animate");
        t.resolve_all();
        assert!(t.obj_now(vault).is(CardType::Creature));
        t.answer_targets(P0, &[Entity::Player(P1)]);
        destroy(&mut t, vault);
        assert_eq!(triggers_on_stack(&t, "dies"), usize::from(!hush), "{hush}");
        t.resolve_all();
    }
}

#[test]
fn hushbringer_leaves_replacement_effects_alone() {
    cr!("614.1c", "614.6", "603.2");
    ruling!(
        "Hushbringer",
        "Replacement effects are unaffected by Hushbringer's ability. For example, a creature that enters the battlefield with one +1/+1 counter on it will still receive a +1/+1 counter, and Leyline of the Void will still exile creatures that would die."
    );
    supported("Servant of the Scale");
    supported("Leyline of the Void");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushbringer");
    t.battlefield(P0, "Leyline of the Void");
    let servant = t.enter(P0, "Servant of the Scale");
    t.settle();
    assert_eq!(t.counters(servant, "+1/+1"), 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn hushbringer_stops_artifact_put_into_a_graveyard_triggers_for_artifact_creatures() {
    cr!("603.6c", "700.4");
    ruling!(
        "Hushbringer",
        "The trigger event doesn't have to specifically refer to \"creatures.\" In these cases, the trigger event may also refer to something being \"put into a graveyard from the battlefield.\" For example, an ability that triggers \"whenever an artifact is put into a graveyard from the battlefield\" won't trigger if an artifact creature dies while Hushbringer is on the battlefield."
    );
    supported("Molder Beast");
    // Molder Beast: "Whenever an artifact is put into a graveyard from the battlefield,
    // this creature gets +2/+0 until end of turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushbringer");
    let beast = t.battlefield(P0, "Molder Beast");
    let thopter = t.battlefield(P1, "Ornithopter");
    destroy(&mut t, thopter);
    t.resolve_all();
    assert_eq!(t.pt(beast), (5, 3));
    // A noncreature artifact: it triggers.
    let rock = t.battlefield(P1, "Mind Stone");
    destroy(&mut t, rock);
    t.resolve_all();
    assert_eq!(t.pt(beast), (7, 3));
}

#[test]
fn entering_together_with_the_hatebear_triggers_nothing() {
    cr!("603.6a", "603.2");
    ruling!(
        "Doorkeeper Thrull",
        "If Doorkeeper Thrull and another creature or an artifact enter the battlefield at the same time, neither one will cause triggered abilities to trigger when they enter the battlefield."
    );
    ruling!(
        "Tocatli Honor Guard",
        "If Tocatli Honor Guard and another creature enter the battlefield at the same time, neither creature entering the battlefield will cause triggered abilities to trigger."
    );
    ruling!(
        "Hushwing Gryff",
        "If Hushwing Gryff and another creature enter the battlefield at the same time, neither creature entering the battlefield will cause triggered abilities to trigger."
    );
    supported("Doorkeeper Thrull");
    supported("Tocatli Honor Guard");
    supported("Ichor Wellspring");
    // Soul Warden: "Whenever another creature enters, you gain 1 life." Elvish
    // Visionary: "When this creature enters, draw a card." Ichor Wellspring: "When this
    // artifact enters ..., draw a card."
    for hatebear in [
        None,
        Some("Doorkeeper Thrull"),
        Some("Tocatli Honor Guard"),
        Some("Hushwing Gryff"),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Soul Warden");
        let hand = t.hand_size(P0);
        let mut cards = vec![("Elvish Visionary", P0)];
        if let Some(h) = hatebear {
            cards.push((h, P0));
        }
        if hatebear == Some("Doorkeeper Thrull") || hatebear.is_none() {
            cards.push(("Ichor Wellspring", P0));
        }
        enter_together(&mut t, &cards);
        t.resolve_all();
        match hatebear {
            None => {
                assert_eq!(t.hand_size(P0), hand + 2);
                assert_eq!(t.life(P0), 21);
            }
            Some(h) => {
                assert_eq!(t.hand_size(P0), hand, "{h}");
                assert_eq!(t.life(P0), 20, "{h}");
            }
        }
    }
}

#[test]
fn doorkeeper_thrull_judges_the_permanent_with_continuous_effects() {
    cr!("603.6a", "613.1d");
    ruling!(
        "Doorkeeper Thrull",
        "Look at the permanent as it exists on the battlefield, taking into account continuous effects, to determine whether any triggered abilities will trigger. For example, if you control Opalescence, which says, in part, \"Each other non-Aura enchantment is a creature,\" each enchantment will be a creature at the time it enters the battlefield and will not cause triggered abilities to trigger."
    );
    supported("Opalescence");
    supported("Ground Seal");
    // Ground Seal: "When this enchantment enters, draw a card."
    for opalescence in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Doorkeeper Thrull");
        if opalescence {
            t.battlefield(P0, "Opalescence");
        }
        let hand = t.hand_size(P0);
        let seal = t.enter(P0, "Ground Seal");
        t.resolve_all();
        assert_eq!(t.obj_now(seal).is(CardType::Creature), opalescence);
        assert_eq!(t.hand_size(P0), if opalescence { hand } else { hand + 1 });
    }
}

#[test]
fn replacement_effects_still_apply_to_permanents_entering() {
    cr!("603.6d", "614.1c", "614.1d", "614.12");
    ruling!(
        "Doorkeeper Thrull",
        "Replacement effects, such as a permanent entering the battlefield tapped or with counters on it, are unaffected. Abilities that apply \"as [this permanent] enters the battlefield\" are replacement effects."
    );
    ruling!(
        "Tocatli Honor Guard",
        "Replacement effects, such as a permanent entering the battlefield tapped or with counters on it, are unaffected. Abilities that apply “as [this creature] enters the battlefield” are replacement effects."
    );
    supported("Diregraf Ghoul");
    supported("Forgotten Sentinel");
    for hatebear in ["Doorkeeper Thrull", "Tocatli Honor Guard"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, hatebear);
        // Diregraf Ghoul: "This creature enters tapped."
        let ghoul = t.enter(P0, "Diregraf Ghoul");
        assert!(tapped(&t, ghoul), "{hatebear}");
        // Servant of the Scale: "This creature enters with a +1/+1 counter on it."
        let servant = t.enter(P0, "Servant of the Scale");
        assert_eq!(t.counters(servant, "+1/+1"), 1, "{hatebear}");
        // Diamond Knight: "As this creature enters, choose a color."
        let k = red_diamond_knight(&mut t);
        t.lands(P0, "Mountain", 1);
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(P1).go();
        t.resolve_all();
        assert_eq!(t.counters(k, "+1/+1"), 1, "{hatebear}");
        // An artifact creature entering tapped (Forgotten Sentinel), under Doorkeeper.
        let sentinel = t.enter(P0, "Forgotten Sentinel");
        assert!(tapped(&t, sentinel), "{hatebear}");
        assert_eq!(t.stack_len(), 0);
    }
}

#[test]
fn doorkeeper_thrull_and_tireless_tracker() {
    cr!("603.6a", "603.2");
    ruling!(
        "Doorkeeper Thrull",
        "The trigger event doesn't have to specify artifacts or creatures entering the battlefield. For example, Tireless Tracker has an ability that says \"Whenever a land enters the battlefield under your control, investigate.\" If an artifact land or creature land enters the battlefield under your control, that ability won't trigger. If a nonartifact, noncreature land enters the battlefield under your control, the ability will trigger."
    );
    supported("Tireless Tracker");
    supported("Seat of the Synod");
    supported("Dryad Arbor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doorkeeper Thrull");
    t.battlefield(P0, "Tireless Tracker");
    let clues = |t: &TestGame| t.named_on_battlefield("Clue Token").len();
    for land in ["Seat of the Synod", "Dryad Arbor"] {
        t.enter(P0, land);
        t.resolve_all();
        assert_eq!(clues(&t), 0, "{land}");
    }
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(clues(&t), 1);
}

#[test]
fn tocatli_honor_guard_and_contraband_kingpin() {
    cr!("603.6a", "603.2");
    ruling!(
        "Tocatli Honor Guard",
        "The trigger event doesn’t have to specify “creatures” entering the battlefield. For example, Contraband Kingpin has an ability that says “Whenever an artifact enters the battlefield under your control, scry 1.” If an artifact creature enters the battlefield under your control, that ability won’t trigger. If a noncreature artifact enters the battlefield under your control, the ability will trigger."
    );
    supported("Contraband Kingpin");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tocatli Honor Guard");
    t.battlefield(P0, "Contraband Kingpin");
    t.enter(P0, "Ornithopter");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.enter(P0, "Mind Stone");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "scry"), 1);
}

#[test]
fn tocatli_honor_guard_stops_its_own_entering_and_enters_abilities() {
    cr!("603.1", "603.6a", "603.2");
    ruling!(
        "Tocatli Honor Guard",
        "Tocatli Honor Guard’s ability stops a creature’s own enters-the-battlefield triggered abilities as well as other triggered abilities that would trigger when a creature enters the battlefield. This includes abilities that would trigger when Tocatli Honor Guard itself enters the battlefield."
    );
    ruling!(
        "Tocatli Honor Guard",
        "Triggered abilities use the word “when,” “whenever,” or “at.” They’re usually written as “[Trigger condition], [effect].”"
    );
    // Tocatli itself entering: Soul Warden doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Tocatli Honor Guard");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Another creature's own "When this creature enters, draw a card." (a triggered
    // ability: "when") doesn't trigger, but Benalish Marshal's static ability ("Other
    // creatures you control get +1/+1.") applies from the moment it enters.
    let hand = t.hand_size(P0);
    let v = t.enter(P0, "Elvish Visionary");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), hand);
    t.enter(P0, "Benalish Marshal");
    t.settle();
    assert_eq!(t.pt(v), (2, 2));
    // A triggered ability with "at" that isn't about entering still triggers.
    t.battlefield(P0, "Phyrexian Arena");
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw"), 1);
}

#[test]
fn hushwing_gryff_and_amulet_of_vigor() {
    cr!("603.6a", "603.2");
    ruling!(
        "Hushwing Gryff",
        "The trigger event doesn’t have to specify “creatures” entering the battlefield. For example, Amulet of Vigor says “Whenever a permanent enters the battlefield tapped and under your control, untap it.” If a creature enters the battlefield tapped and under your control, Amulet of Vigor’s ability would not trigger. If a land (that isn’t also a creature) enters the battlefield tapped and under your control, Amulet of Vigor’s ability would trigger."
    );
    supported("Amulet of Vigor");
    supported("Tranquil Cove");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushwing Gryff");
    t.battlefield(P0, "Amulet of Vigor");
    let ghoul = t.enter(P0, "Diregraf Ghoul");
    t.resolve_all();
    assert!(tapped(&t, ghoul));
    let cove = t.enter(P0, "Tranquil Cove");
    t.resolve_all();
    assert!(!tapped(&t, cove));
}

#[test]
fn containment_priest_doesnt_affect_creatures_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Containment Priest",
        "If Containment Priest enters the battlefield at the same time as other creatures, its ability won't affect those creatures."
    );
    supported("Containment Priest");
    let mut t = TestGame::new(2);
    let ids = enter_together(&mut t, &[("Containment Priest", P0), ("Grizzly Bears", P0)]);
    t.settle();
    assert_eq!(t.zone(ids[1]), Zone::Battlefield);
    // Once it's on the battlefield, a creature put onto it without being cast is exiled.
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.g.move_object(
        bears,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    t.g.flush_events();
    assert_eq!(t.zone(bears), Zone::Exile);
}
