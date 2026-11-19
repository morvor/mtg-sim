//! Rulings batch P083 — enters-the-battlefield hate (Torpor Orb, Mistcaller; CR 603.6a,
//! 614.12, 603.2g), target changing (Muck Drubb; CR 115.7), Auras and Equipment
//! (Azorius First-Wing, Aura Barbs, Fulgent Distraction, Artificer's Hex; CR 701.3,
//! 702.16c), and Tamiyo, Collector of Tales (CR 606.4, 701.21, 704.5).

use crate::r_p008_common::damage_events;
use crate::r_p083_common::*;
use crate::r_s01_common::tokens;
use crate::r_s06_common::{attach_new, attached_to};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------
// Torpor Orb: "Creatures entering don't cause abilities to trigger."
// ---------------------------------------------------------------------------------

#[test]
fn torpor_orb_doesnt_stop_as_enters_choices_or_enters_with_counters() {
    cr!("614.1c", "614.12", "603.6d");
    ruling!(
        "Torpor Orb",
        "Abilities that apply \"as [this creature] enters the battlefield,\" including many abilities that ask a creature's controller to choose a color or creature type, are unaffected."
    );
    ruling!(
        "Torpor Orb",
        "Replacement effects are unaffected by Torpor Orb's ability. For example, a creature that enters the battlefield with one +1/+1 counter on it will still receive a +1/+1 counter."
    );
    supported("Torpor Orb");
    supported("Painter's Servant");
    supported("Mistcutter Hydra");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Torpor Orb");
    let bolt = t.hand(P0, "Lightning Bolt");
    crate::r_p022_common::choose_color(&mut t, P0, Color::Green);
    t.enter(P0, "Painter's Servant");
    t.settle();
    t.g.recompute();
    assert!(
        t.obj_now(bolt).chars.colors.contains(Color::Green),
        "the color was chosen"
    );
    // Mistcutter Hydra with X = 2 enters with two +1/+1 counters.
    t.lands(P0, "Forest", 3);
    let c = t.hand(P0, "Mistcutter Hydra");
    t.cast(P0, c).x(2).go();
    t.resolve_all();
    let hydra = t.named_on_battlefield("Mistcutter Hydra")[0];
    assert_eq!(t.counters(hydra, "+1/+1"), 2);
}

#[test]
fn torpor_orb_entering_with_a_creature() {
    cr!("603.6a", "603.2", "603.10");
    ruling!(
        "Torpor Orb",
        "If Torpor Orb and a creature enter the battlefield at the same time, that creature entering the battlefield won't cause triggered abilities to trigger."
    );
    supported("Open the Vaults");
    supported("Solemn Simulacrum");
    // Open the Vaults returns Torpor Orb and Solemn Simulacrum together: no search.
    let lands = |t: &TestGame| t.g.permanents().filter(|o| o.is(CardType::Land)).count();
    for orb in [true, false] {
        let mut t = TestGame::new(2);
        if orb {
            t.graveyard(P0, "Torpor Orb");
        }
        t.graveyard(P0, "Solemn Simulacrum");
        t.library_top(P0, "Forest");
        cast_new(&mut t, P0, "Open the Vaults", &[]);
        let before = lands(&t);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Solemn Simulacrum").len(), 1);
        assert_eq!(t.named_on_battlefield("Torpor Orb").len(), usize::from(orb));
        assert_eq!(
            lands(&t),
            before + usize::from(!orb),
            "orb = {orb}: the search"
        );
    }
}

#[test]
fn torpor_orb_looks_at_the_permanent_as_it_enters() {
    cr!("603.6a", "603.10", "613.1d");
    ruling!(
        "Torpor Orb",
        "Look at the permanent as it exists on the battlefield, taking into account continuous effects, to determine whether any triggered abilities will trigger. For example, if you control March of the Machines, which says, in part, \"Each noncreature artifact is an artifact creature,\" each artifact will be a creature at the time it enters the battlefield and will not cause triggered abilities to trigger."
    );
    ruling!(
        "Torpor Orb",
        "The trigger event doesn't have to specify \"creatures\" entering the battlefield. For example, Captain Storm, Cosmium Raider from the Lost Caverns of Ixalan release has an ability that says \"Whenever an artifact enters the battlefield under your control, put a +1/+1 counter on target Pirate you control.\" If an artifact creature enters the battlefield under your control, that ability won't trigger. If a noncreature artifact enters the battlefield under your control, the ability will trigger."
    );
    supported("March of the Machines");
    supported("Ichor Wellspring");
    supported("Captain Storm, Cosmium Raider");
    // Ichor Wellspring ("When this artifact enters ..., draw a card") with March of the
    // Machines: it's a creature as it enters, so it doesn't draw.
    for march in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Torpor Orb");
        if march {
            t.battlefield(P0, "March of the Machines");
        }
        let hand = t.hand_size(P0);
        t.enter(P0, "Ichor Wellspring");
        t.resolve_all();
        assert_eq!(
            t.hand_size(P0),
            hand + usize::from(!march),
            "march = {march}"
        );
    }
    // Captain Storm: an artifact creature entering doesn't trigger it; a noncreature
    // artifact does.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Torpor Orb");
    let storm = t.battlefield(P0, "Captain Storm, Cosmium Raider");
    t.enter(P0, "Solemn Simulacrum");
    t.resolve_all();
    assert_eq!(t.counters(storm, "+1/+1"), 0);
    t.enter(P0, "Sol Ring");
    t.resolve_all();
    assert_eq!(t.counters(storm, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------------
// Mistcaller: "Sacrifice this creature: Until end of turn, if a nontoken creature would
// enter and it wasn't cast, exile it instead."
// ---------------------------------------------------------------------------------

/// P0's Mistcaller, sacrificed (its ability resolves).
fn mistcaller(t: &mut TestGame) {
    supported("Mistcaller");
    let m = t.battlefield(P0, "Mistcaller");
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mistcaller"));
}

#[test]
fn mistcaller_entering_with_other_creatures_is_too_late() {
    cr!("602.2", "614.1a", "614.12");
    ruling!(
        "Mistcaller",
        "If Mistcaller enters the battlefield at the same time as other creatures, its ability can't be activated in time to affect those creatures."
    );
    supported("Living Death");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Mistcaller");
    t.graveyard(P0, "Craw Wurm");
    cast_resolve(&mut t, P0, "Living Death", &[]);
    let m = t.named_on_battlefield("Mistcaller");
    assert_eq!(m.len(), 1);
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    // Activating it now doesn't touch the Wurm.
    t.activate(P0, m[0], 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn mistcaller_ignores_tokens_permanents_and_cast_creatures() {
    cr!("614.1a", "601.2", "111.1");
    ruling!(
        "Mistcaller",
        "Mistcaller’s ability doesn’t stop creature tokens from entering the battlefield. It also doesn’t affect creatures that are already on the battlefield."
    );
    ruling!(
        "Mistcaller",
        "Mistcaller’s ability won’t affect any nontoken creatures that were cast, including ones cast from unusual zones such as your graveyard."
    );
    supported("Raise the Alarm");
    supported("Gravecrawler");
    supported("Zombify");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Scathe Zombies");
    let craw = t.graveyard(P0, "Craw Wurm");
    let crawler = t.graveyard(P0, "Gravecrawler");
    mistcaller(&mut t);
    assert!(t.on_battlefield(bears));
    // Tokens enter.
    cast_resolve(&mut t, P0, "Raise the Alarm", &[]);
    assert_eq!(tokens(&t, P0).len(), 2);
    // Cast creatures enter, also from the graveyard (Gravecrawler, controlling a Zombie).
    cast_resolve(&mut t, P0, "Llanowar Elves", &[]);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    t.lands(P0, "Swamp", 1);
    t.cast(P0, crawler).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gravecrawler").len(), 1);
    // A creature put onto the battlefield without being cast is exiled.
    cast_resolve(&mut t, P0, "Zombify", &[obj(craw)]);
    assert!(t.in_exile("Craw Wurm"));
}

// ---------------------------------------------------------------------------------
// Muck Drubb: "When this creature enters, change the target of target spell that targets
// only a single creature to this creature."
// ---------------------------------------------------------------------------------

#[test]
fn muck_drubb_cant_target_a_spell_with_several_targets() {
    cr!("115.9c", "603.3d");
    ruling!(
        "Muck Drubb",
        "If a spell has multiple targets, you can't target it with Muck Drubb's ability, even if only one of those targets is a creature or all but one of those targets have become illegal."
    );
    supported("Muck Drubb");
    supported("Electrolyze");
    // Electrolyze targeting a creature and a player: one creature, but two targets.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P1, "Electrolyze");
    let el = t.hand(P1, "Electrolyze");
    t.answer(
        P1,
        DecisionKind::Divide,
        mtg_engine::decision::Answer::Numbers(vec![1, 1]),
    );
    t.cast(P1, el).targets(&[obj(bears), player(P0)]).go();
    t.enter(P0, "Muck Drubb");
    t.settle();
    assert_eq!(t.stack_len(), 1, "no legal target: the trigger is removed");
    t.resolve_all();
    assert_eq!(damage_on(&t, bears), 1);
    // Two creatures, one gone since: still two targets.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    crate::r_s25_common::lands_for_cost(&mut t, P1, "Electrolyze");
    let el = t.hand(P1, "Electrolyze");
    t.answer(
        P1,
        DecisionKind::Divide,
        mtg_engine::decision::Answer::Numbers(vec![1, 1]),
    );
    t.cast(P1, el).targets(&[obj(bears), obj(wurm)]).go();
    cast_new(&mut t, P0, "Unsummon", &[obj(bears)]);
    t.resolve();
    t.enter(P0, "Muck Drubb");
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn muck_drubb_changes_the_target_only_to_a_legal_one() {
    cr!("115.7", "115.7a");
    ruling!(
        "Muck Drubb",
        "The spell's target is changed to Muck Drubb only if Muck Drubb is a legal target for that spell."
    );
    // Doom Blade ("Destroy target nonblack creature") can't target the black Drubb.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = cast_new(&mut t, P1, "Doom Blade", &[obj(bears)]);
    t.answer_targets(P0, &[obj(blade)]);
    t.enter(P0, "Muck Drubb");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Muck Drubb").len(), 1);
    // Lightning Bolt is redirected to it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[obj(bears)]);
    t.answer_targets(P0, &[obj(bolt)]);
    t.enter(P0, "Muck Drubb");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_on(&t, bears), 0);
    assert!(
        t.in_graveyard(P0, "Muck Drubb"),
        "3 damage to the 3/3 Drubb"
    );
}

// ---------------------------------------------------------------------------------
// Auras and Equipment.
// ---------------------------------------------------------------------------------

#[test]
fn an_aura_cant_be_moved_onto_azorius_first_wing() {
    cr!("701.3b", "303.4j", "702.16c");
    ruling!(
        "Azorius First-Wing",
        "If an effect tries to move an Aura onto this creature, the effect will fail and the Aura will stay where it was."
    );
    supported("Azorius First-Wing");
    supported("Simic Guildmage");
    // Simic Guildmage: "{1}{U}: Attach target Aura attached to a permanent to another
    // permanent with the same controller."
    // P1's Guildmage moves P1's Pacifism from P0's Bears: the only other permanent P0
    // controls is the First-Wing, which the Aura can't be attached to.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wing = t.battlefield(P0, "Azorius First-Wing");
    let aura = attach_new(&mut t, P1, "Pacifism", bears);
    let gm = t.battlefield(P1, "Simic Guildmage");
    t.lands(P1, "Island", 2);
    t.answer_choose(P1, &[obj(wing)]);
    crate::r_s06_common::activate_containing(&mut t, P1, gm, "Attach target Aura").unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, aura), Some(obj(bears)), "the Aura stays");
    assert!(t.on_battlefield(aura));
}

#[test]
fn aura_barbs_damage_is_dealt_by_the_enchantments() {
    cr!("120.3", "702.16e");
    ruling!(
        "Aura Barbs",
        "The damage is dealt by the enchantments, not by Aura Barbs."
    );
    supported("Aura Barbs");
    supported("Kor Firewalker");
    // P1's Pacifism on P0's Kor Firewalker (protection from red): the white Pacifism, not
    // the red Aura Barbs, deals the damage, so it isn't prevented.
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P0, "Kor Firewalker");
    let pacifism = attach_new(&mut t, P1, "Pacifism", walker);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let from = t.g.turn_events.len();
    cast_resolve(&mut t, P0, "Aura Barbs", &[]);
    assert_eq!(t.life(P1), 16, "each of P1's enchantments dealt 2 to P1");
    assert!(t.in_graveyard(P0, "Kor Firewalker"));
    let sources: Vec<ObjectId> = damage_events(&t, from).into_iter().map(|d| d.0).collect();
    assert_eq!(sources.len(), 3);
    assert!(sources.iter().all(|s| *s == pacifism || *s == anthem));
}

/// P0 casts Fulgent Distraction ("Choose two target creatures.") at `a` and `b`.
fn distraction(t: &mut TestGame, a: ObjectId, b: ObjectId) {
    crate::r_s25_common::lands_for_cost(t, P0, "Fulgent Distraction");
    let c = t.hand(P0, "Fulgent Distraction");
    let spell = t.cast(P0, c).targets(&[obj(a), obj(b)]).go();
    assert_eq!(
        crate::r_s25_common::targets_of(t, spell),
        vec![obj(a), obj(b)]
    );
}

#[test]
fn fulgent_distraction_needs_two_targets_and_does_what_it_can() {
    cr!("601.2c", "608.2b", "701.3d");
    ruling!(
        "Fulgent Distraction",
        "Fulgent Distraction can target tapped and/or unequipped creatures."
    );
    ruling!(
        "Fulgent Distraction",
        "If one of the targeted creatures is an illegal target by the time Fulgent Distraction resolves, that creature won’t become tapped and no Equipment will be unattached from it. The other creature will still be affected. If both of the targeted creatures are illegal targets by the time Fulgent Distraction resolves, the spell doesn’t resolve."
    );
    ruling!(
        "Fulgent Distraction",
        "You must target two creatures as you cast Fulgent Distraction. If you can’t (because there’s just one creature on the battlefield, perhaps), you can’t cast the spell."
    );
    supported("Fulgent Distraction");
    // A tapped, unequipped creature and an equipped one.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.g.obj_mut(a).tapped = true;
    let b = t.battlefield(P1, "Craw Wurm");
    let bone = attach_new(&mut t, P1, "Bonesplitter", b);
    distraction(&mut t, a, b);
    t.resolve_all();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert_eq!(attached_to(&t, bone), None);
    // One target gone: the other is still tapped and unequipped.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Craw Wurm");
    let bone = attach_new(&mut t, P1, "Bonesplitter", b);
    let bone2 = attach_new(&mut t, P1, "Bonesplitter", a);
    distraction(&mut t, a, b);
    cast_resolve(&mut t, P1, "Unsummon", &[obj(a)]);
    assert!(t.obj_now(b).tapped);
    assert_eq!(attached_to(&t, bone), None);
    assert!(t.on_battlefield(bone2));
    // Both gone: it doesn't resolve.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Craw Wurm");
    distraction(&mut t, a, b);
    crate::r_p146_common::destroy_together(&mut t, &[a, b]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fulgent Distraction"));
    // Only one creature: it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Fulgent Distraction");
    let c = t.hand(P0, "Fulgent Distraction");
    assert!(!crate::r_s21_common::castable(&mut t, P0, c));
    t.battlefield(P0, "Llanowar Elves");
    assert!(crate::r_s21_common::castable(&mut t, P0, c));
}

#[test]
fn artificers_hex_equip_cant_unattach() {
    cr!("702.6a", "701.3b", "603.4");
    ruling!(
        "Artificer's Hex",
        "Players can’t pay an Equipment’s equip cost to unattach that Equipment to stop Artificer’s Hex from destroying a creature."
    );
    supported("Artificer's Hex");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bone = attach_new(&mut t, P0, "Bonesplitter", bears);
    attach_new(&mut t, P1, "Artificer's Hex", bone);
    // Equip targets a creature to attach to: "attaching" it to the creature it's already
    // attached to does nothing, and there's nothing else to target.
    let targets = ability_targets(&mut t, bone, 0);
    assert_eq!(targets, vec![obj(bears)]);
    t.lands(P0, "Wastes", 1);
    t.activate(P0, bone, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, bone), Some(obj(bears)));
    // P1's upkeep: the Hex's trigger destroys the Bears.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------------
// Tamiyo, Collector of Tales.
// ---------------------------------------------------------------------------------

#[test]
fn tamiyo_with_three_loyalty_cant_return_herself() {
    cr!("606.4", "601.2c", "602.2b", "704.5i");
    ruling!(
        "Tamiyo, Collector of Tales",
        "If Tamiyo has 3 loyalty, you can’t activate her last ability to return her to your hand. She won’t be in your graveyard while you’re choosing targets for the ability."
    );
    let mut t = TestGame::new(2);
    let tamiyo = t.battlefield(P0, "Tamiyo, Collector of Tales");
    t.g.obj_mut(tamiyo).counters.insert("loyalty".into(), 3);
    // An empty graveyard: no legal target, so the −3 can't be activated.
    assert!(t.activate(P0, tamiyo, 1, &[]).is_err());
    assert!(t.on_battlefield(tamiyo));
    // With another card there, it targets that card; Tamiyo dies afterwards.
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.activate(P0, tamiyo, 1, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Tamiyo, Collector of Tales"));
}

#[test]
fn tamiyo_stops_sacrifices_not_game_rules() {
    cr!("701.21a", "704.5f", "704.5g", "704.5j");
    ruling!(
        "Tamiyo, Collector of Tales",
        "Tamiyo’s ability affects sacrifices, but not any other ways permanents can leave the battlefield. It won’t stop a creature from dying due to lethal damage or having 0 toughness, and it won’t stop a permanent from being put into its owner’s graveyard due to the “legend rule.” None of these are sacrifices; they’re the result of game rules."
    );
    supported("Diabolic Edict");
    supported("Disfigure");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tamiyo, Collector of Tales");
    let wurm = t.battlefield(P0, "Craw Wurm");
    // P1's Diabolic Edict: no sacrifice.
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Diabolic Edict", &[player(P0)]);
    assert!(t.on_battlefield(wurm));
    // Lethal damage, 0 toughness.
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P1, "Disfigure", &[obj(bears)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let elves = t.battlefield(P0, "Llanowar Elves");
    cast_resolve(&mut t, P1, "Lightning Bolt", &[obj(elves)]);
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    // The legend rule.
    t.battlefield(P0, "Tymaret, Chosen from Death");
    t.battlefield(P0, "Tymaret, Chosen from Death");
    t.settle();
    assert_eq!(
        t.named_on_battlefield("Tymaret, Chosen from Death").len(),
        1
    );
    assert!(t.in_graveyard(P0, "Tymaret, Chosen from Death"));
}
