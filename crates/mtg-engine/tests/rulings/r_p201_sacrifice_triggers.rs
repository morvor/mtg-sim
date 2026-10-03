//! Rulings batch P201 — "whenever you/a player sacrifice(s) ..." triggers (CR 701.21,
//! 603.2, 603.3, 603.10a): Bloodbriar, Havoc Jester, Mayhem Devil, Mortician Beetle,
//! Ulvenwald Mysteries, Dragon Appeasement, Furnace Celebration, Sanguine Brushstroke,
//! Moonstone Eulogist, Merchant of Venom, Zodiark, Ashad.

use crate::r_s01_common::{supported, tokens};
use crate::r_s02_common::create_token;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Decision;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The triggered abilities from `src` on the stack.
fn triggers_from(t: &TestGame, src: ObjectId) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_ref().is_some_and(
                |si| matches!(&si.kind, StackKind::Triggered { source, .. } if *source == src),
            )
        })
        .count()
}

/// Whether the top object of the stack is a triggered ability from `src`.
fn top_is_trigger_from(t: &TestGame, src: ObjectId) -> bool {
    t.g.stack.last().is_some_and(|id| {
        t.g.obj(*id).stack.as_ref().is_some_and(
            |si| matches!(&si.kind, StackKind::Triggered { source, .. } if *source == src),
        )
    })
}

/// `p` sacrifices the permanents at the same time (as a resolving effect instructs), then
/// state-based actions are checked and triggers are put on the stack.
fn sacrifice(t: &mut TestGame, p: PlayerId, ids: &[ObjectId]) {
    let what: Vec<(ObjectId, PlayerId)> = ids.iter().map(|i| (*i, p)).collect();
    t.g.sacrifice_simultaneously(&what);
    t.settle();
}

fn has_activated_ability(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Activated(_)))
}

/// P0 casts Altar's Reap ("As an additional cost to cast this spell, sacrifice a
/// creature. Draw two cards.") sacrificing `victim`; returns the spell.
fn altars_reap(t: &mut TestGame, victim: ObjectId) -> ObjectId {
    supported("Altar's Reap");
    t.lands(P0, "Swamp", 2);
    let reap = t.hand(P0, "Altar's Reap");
    t.answer_choose(P0, &[Entity::Object(victim)]);
    let spell = t.cast(P0, reap).go();
    t.settle();
    spell
}

#[test]
fn sacrifice_triggers_are_not_sacrifice_outlets() {
    cr!("603.1", "701.21a");
    ruling!(
        "Bloodbriar",
        "Bloodbriar's ability is a triggered ability, not an activated ability."
    );
    ruling!(
        "Havoc Jester",
        "Havoc Jester's ability is a triggered ability, not an activated ability."
    );
    ruling!(
        "Ulvenwald Mysteries",
        "The last ability is a triggered ability, not an activated ability. It doesn’t allow you to sacrifice a Clue whenever you want"
    );
    ruling!(
        "Dragon Appeasement",
        "Dragon Appeasement itself doesn’t allow you to sacrifice any creatures."
    );
    ruling!(
        "Furnace Celebration",
        "Furnace Celebration itself doesn't allow you to sacrifice any permanents."
    );
    ruling!(
        "Mayhem Devil",
        "Mayhem Devil itself doesn't allow any player to sacrifice any permanents."
    );
    ruling!(
        "Mortician Beetle",
        "Mortician Beetle’s ability triggers whenever any player, including you, sacrifices a creature"
    );
    // (card, whose sacrifice triggers it, whether the sacrificed permanent is a Clue)
    let cases: &[(&str, PlayerId, bool)] = &[
        ("Bloodbriar", P0, false),
        ("Havoc Jester", P0, false),
        ("Ulvenwald Mysteries", P0, true),
        ("Dragon Appeasement", P0, false),
        ("Furnace Celebration", P0, false),
        ("Mayhem Devil", P0, false),
        ("Mayhem Devil", P1, false),
        ("Mortician Beetle", P0, false),
        ("Mortician Beetle", P1, false),
    ];
    for &(name, who, clue) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        assert!(!has_activated_ability(&t, src), "{name}");
        let victim = if clue {
            create_token(&mut t, who, "Clue")
        } else {
            t.battlefield(who, "Grizzly Bears")
        };
        t.settle();
        assert_eq!(triggers_from(&t, src), 0);
        // Something else makes the player sacrifice it: the ability triggers.
        sacrifice(&mut t, who, &[victim]);
        assert_eq!(triggers_from(&t, src), 1, "{name} ({who:?})");
    }
}

#[test]
fn the_legend_rule_doesnt_sacrifice() {
    cr!("704.5j", "701.21a");
    ruling!(
        "Bloodbriar",
        "A legendary permanent that is put into a graveyard because of the “legend rule” isn't sacrificed."
    );
    for name in ["Bloodbriar", "Havoc Jester"] {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        t.battlefield(P0, "Isamaru, Hound of Konda");
        t.battlefield(P0, "Isamaru, Hound of Konda");
        t.settle();
        assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
        assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
        assert_eq!(triggers_from(&t, src), 0, "{name}");
    }
}

#[test]
fn sacrificing_as_a_cost_triggers_before_the_spell_resolves() {
    cr!("601.2h", "603.3", "117.3c");
    ruling!(
        "Bloodbriar",
        "If you sacrifice a permanent as part of casting a spell or activating an ability, Bloodbriar's ability will resolve before that spell or ability."
    );
    ruling!(
        "Havoc Jester",
        "If you sacrifice a permanent as part of casting a spell or activating an ability, Havoc Jester's ability will resolve before that spell or ability."
    );
    ruling!(
        "Mortician Beetle",
        "If a creature is sacrificed as a cost to cast a spell or activate an ability, Mortician Beetle’s ability resolves before that spell or ability."
    );
    ruling!(
        "Mayhem Devil",
        "If a permanent is sacrificed to pay a cost of a spell or ability, Mayhem Devil's ability will resolve before that spell or ability."
    );
    for name in [
        "Bloodbriar",
        "Havoc Jester",
        "Mortician Beetle",
        "Mayhem Devil",
    ] {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_yes(P0, true);
        let reap = altars_reap(&mut t, bears);
        assert!(top_is_trigger_from(&t, src), "{name}");
        let hand = t.hand_size(P0);
        // The trigger resolves first; the spell is still on the stack.
        t.resolve();
        assert!(t.g.stack.contains(&reap));
        assert_eq!(t.hand_size(P0), hand);
        match name {
            "Bloodbriar" | "Mortician Beetle" => assert_eq!(t.counters(src, "+1/+1"), 1),
            _ => assert_eq!(t.life(P1), 19),
        }
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + 2);
    }
}

#[test]
fn sacrificing_a_clue_to_activate_it_triggers_before_the_clue_resolves() {
    cr!("602.2b", "603.3", "701.16a");
    ruling!(
        "Ulvenwald Mysteries",
        "If you sacrifice a Clue as part of casting a spell or activating an ability, the last ability will resolve before that spell or ability."
    );
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Ulvenwald Mysteries");
    let clue = create_token(&mut t, P0, "Clue");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, clue, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(top_is_trigger_from(&t, src));
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.hand_size(P0), hand);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn mayhem_devil_triggers_after_a_resolving_sacrifice_and_for_itself() {
    cr!("603.3", "608.2", "603.10a");
    ruling!(
        "Mayhem Devil",
        "Conversely, if a permanent is sacrificed during the resolution of a spell or ability, that spell or ability will finish resolving before Mayhem Devil's ability is put onto the stack."
    );
    ruling!(
        "Mayhem Devil",
        "If you sacrifice Mayhem Devil, its ability triggers."
    );
    ruling!(
        "Mayhem Devil",
        "You control Mayhem Devil's triggered ability and choose the target, no matter who sacrificed the permanent."
    );
    supported("Diabolic Edict");
    // P0's Diabolic Edict ("Target player sacrifices a creature of their choice.") makes
    // P1 sacrifice; the Edict finishes resolving first. P0 controls the trigger and
    // chooses its target.
    let mut t = TestGame::new(2);
    let devil = t.battlefield(P0, "Mayhem Devil");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let edict = t.hand(P0, "Diabolic Edict");
    let edict = t.cast(P0, edict).target(Entity::Player(P1)).go();
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(!t.g.stack.contains(&edict));
    assert!(t.in_graveyard(P0, "Diabolic Edict"));
    assert_eq!(triggers_from(&t, devil), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).controller, P0);
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. })));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseTargets { .. })));
    t.resolve();
    assert_eq!(t.life(P1), 19);
    // Sacrificing Mayhem Devil itself triggers it.
    let mut t = TestGame::new(2);
    let devil = t.battlefield(P0, "Mayhem Devil");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    sacrifice(&mut t, P0, &[devil]);
    assert_eq!(triggers_from(&t, devil), 1);
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn havoc_jester_triggers_for_itself_and_permanents_sacrificed_with_it() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Havoc Jester",
        "Havoc Jester's ability triggers when you sacrifice it. If you sacrifice other permanents at the same time, it triggers for them as well."
    );
    let mut t = TestGame::new(2);
    let jester = t.battlefield(P0, "Havoc Jester");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Forest");
    sacrifice(&mut t, P0, &[jester, a, b]);
    assert_eq!(triggers_from(&t, jester), 3);
    // Alone, too.
    let mut t = TestGame::new(2);
    let jester = t.battlefield(P0, "Havoc Jester");
    sacrifice(&mut t, P0, &[jester]);
    assert_eq!(triggers_from(&t, jester), 1);
}

#[test]
fn moonstone_eulogist_sees_creatures_dying_with_it() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Moonstone Eulogist",
        "If Moonstone Eulogist and one or more creatures opponents control die at the same time, its second ability will trigger for each of those creatures opponents controlled that died."
    );
    supported("Moonstone Eulogist");
    supported("Day of Judgment");
    let mut t = TestGame::new(2);
    let eulogist = t.battlefield(P0, "Moonstone Eulogist");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let doj = t.hand(P0, "Day of Judgment");
    t.cast(P0, doj).go();
    t.resolve();
    assert_eq!(triggers_from(&t, eulogist), 2);
    t.resolve_all();
    let blood: Vec<_> = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype("Blood"))
        .collect();
    assert_eq!(blood.len(), 2);
}

#[test]
fn ulvenwald_mysteries_sees_creatures_dying_with_it_and_itself_as_a_creature() {
    cr!("603.10a", "613.1d");
    ruling!(
        "Ulvenwald Mysteries",
        "If a nontoken creature dies at the same time as Ulvenwald Mysteries leaves the battlefield, the first ability triggers."
    );
    supported("Opalescence");
    // Opalescence: "Each other non-Aura enchantment is a creature in addition to its other
    // types and has base power and base toughness each equal to its mana value."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Opalescence");
    let mysteries = t.battlefield(P0, "Ulvenwald Mysteries");
    t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(mysteries), (3, 3));
    t.lands(P0, "Plains", 4);
    let doj = t.hand(P0, "Day of Judgment");
    t.cast(P0, doj).go();
    t.resolve();
    // Opalescence is a non-creature for itself, so it survives; the Bears and the
    // Mysteries (a nontoken creature) died together: two investigate triggers.
    assert!(!t.on_battlefield(mysteries));
    assert_eq!(triggers_from(&t, mysteries), 2);
    t.resolve_all();
    let clues = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype("Clue"))
        .count();
    assert_eq!(clues, 2);
}

#[test]
fn sanguine_brushstroke_triggers_on_any_blood_sacrifice() {
    cr!("603.2", "701.21a");
    ruling!(
        "Sanguine Brushstroke",
        "Sanguine Brushstroke’s last ability triggers whenever you sacrifice a Blood token, not just when you activate a Blood token’s activated ability."
    );
    supported("Sanguine Brushstroke");
    supported("Deadly Dispute");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Sanguine Brushstroke");
    let blood = create_token(&mut t, P0, "Blood");
    let other = create_token(&mut t, P0, "Clue");
    // Deadly Dispute: "As an additional cost to cast this spell, sacrifice an artifact or
    // creature. Draw two cards and create a Treasure token."
    t.lands(P0, "Swamp", 2);
    let dd = t.hand(P0, "Deadly Dispute");
    t.answer_choose(P0, &[Entity::Object(blood)]);
    t.cast(P0, dd).go();
    t.settle();
    assert_eq!(triggers_from(&t, src), 1);
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
    t.resolve_all();
    // Sacrificing a Clue doesn't trigger it.
    sacrifice(&mut t, P0, &[other]);
    assert_eq!(triggers_from(&t, src), 0);
}

#[test]
fn furnace_celebration_pays_once_on_resolution_if_the_target_is_legal() {
    cr!("603.5", "608.2b", "117.1");
    ruling!(
        "Furnace Celebration",
        "You can't pay {2} more than once each time Furnace Celebration's ability resolves."
    );
    ruling!(
        "Furnace Celebration",
        "You choose whether to pay {2} as the ability resolves, if that target is still legal."
    );
    supported("Furnace Celebration");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Furnace Celebration");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 6);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    sacrifice(&mut t, P0, &[bears]);
    assert_eq!(triggers_from(&t, src), 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    let tapped =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.tapped)
            .count();
    assert_eq!(tapped, 2);
    t.clear_answers();
    // The target becomes illegal: the ability doesn't resolve and nothing is paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Furnace Celebration");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let target = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    t.answer_targets(P0, &[Entity::Object(target)]);
    sacrifice(&mut t, P0, &[bears]);
    t.g.destroy(target, None);
    let from = t.asked().len();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert!(!t
        .asked()
        .iter()
        .skip(from)
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    assert!(t.g.permanents().all(|o| !o.tapped));
}

/// The entity choices asked since `from`, in order, by player.
fn choosers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect()
}

#[test]
fn each_player_sacrifices_choices_are_made_in_apnap_order_then_sacrificed_together() {
    cr!("101.4", "701.21a", "603.2c");
    ruling!(
        "Merchant of Venom",
        "When Merchant of Venom's second ability resolves, first the player whose turn it is chooses which creature they're going to sacrifice, then each other player in turn order does the same."
    );
    ruling!(
        "Zodiark, Umbral God",
        "While resolving Zodiark's second ability, you choose half the non-God creatures you control (rounded down), then each other player in turn order does the same."
    );
    supported("Merchant of Venom");
    supported("Zodiark, Umbral God");
    // Merchant of Venom: "When this creature enters, each player sacrifices a creature of
    // their choice. Whenever a player sacrifices a permanent, put a +1/+1 counter on this
    // creature." Three players: P1 is active, so P1 chooses first, then P2, then P0.
    let mut t = TestGame::new(3);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let bears0 = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears0)]);
    let merchant = t.enter(P0, "Merchant of Venom");
    let from = t.asked().len();
    t.resolve();
    assert_eq!(choosers(&t, from), vec![P1, P2, P0]);
    assert!(t.on_battlefield(merchant));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
    // Three sacrifices at once: three triggers.
    assert_eq!(triggers_from(&t, merchant), 3);
    t.resolve_all();
    assert_eq!(t.counters(merchant, "+1/+1"), 3);
    // Zodiark: "each player sacrifices half the non-God creatures they control of their
    // choice, rounded down. Whenever a player sacrifices another creature, put a +1/+1
    // counter on Zodiark." P0 (active) has three, P1 four.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    for _ in 0..4 {
        t.battlefield(P1, "Grizzly Bears");
    }
    let z = t.enter(P0, "Zodiark, Umbral God");
    let from = t.asked().len();
    t.resolve();
    assert_eq!(choosers(&t, from), vec![P0, P1]);
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.name == "Grizzly Bears")
            .count(),
        2
    );
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P1 && o.chars.name == "Grizzly Bears")
            .count(),
        2
    );
    assert_eq!(triggers_from(&t, z), 3);
    t.resolve_all();
    assert_eq!(t.counters(z, "+1/+1"), 3);
}

#[test]
fn ashad_gives_the_first_nonlegendary_artifact_spell_casualty_2() {
    cr!("702.153a", "707.10f", "111.1");
    ruling!(
        "Ashad, the Lone Cyberman",
        "Casualty 2 means \"As an additional cost to cast this spell, you may sacrifice a creature with power 2 or greater.\""
    );
    supported("Ashad, the Lone Cyberman");
    let mut t = TestGame::new(2);
    let ashad = t.battlefield(P0, "Ashad, the Lone Cyberman");
    let giant = t.battlefield(P0, "Hill Giant");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let thopter = t.hand(P0, "Ornithopter");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let from = t.asked().len();
    t.cast(P0, thopter).go();
    // Only creatures with power 2 or greater (Ashad and the Hill Giant) could be
    // sacrificed, not the 1/1 Raging Goblin.
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&Entity::Object(giant)));
    assert!(offered[0].contains(&Entity::Object(ashad)));
    assert!(!offered[0].contains(&Entity::Object(goblin)));
    assert!(!t.on_battlefield(giant));
    // "When you cast this spell, if a casualty cost was paid for it, copy it": the copy of
    // an artifact spell becomes a token. Ashad also sees the sacrifice.
    t.resolve_all();
    let thopters = t.named_on_battlefield("Ornithopter");
    assert_eq!(thopters.len(), 2);
    assert_eq!(thopters.iter().filter(|i| t.obj(**i).is_token()).count(), 1);
    assert_eq!(t.counters(ashad, "+1/+1"), 1);
}
