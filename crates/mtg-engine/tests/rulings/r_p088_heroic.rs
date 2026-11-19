//! Rulings batch P088 — abilities that trigger when a spell or ability targets a
//! permanent, or when a spell is cast (heroic and its kin, Orvar, King of the
//! Oathbreakers, Battlefield Thaumaturge's cost reduction; CR 603.2, 603.3, 601.2f,
//! 702.26, 707.2).

use crate::r_p088_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The top item of the stack is a triggered ability (by its text), above the spell or
/// ability below it.
fn trigger_on_top(t: &TestGame, text: &str) {
    let items = stack_items(t);
    assert!(items.len() >= 2, "{items:?}");
    assert!(items.last().unwrap().contains(text), "{items:?}");
}

/// P1 counters the topmost spell or ability below the top of the stack (the one that
/// caused the trigger) with Counterspell (a spell) or Stifle (an ability); it resolves.
fn counter_below_top(t: &mut TestGame, ability: bool) {
    let target = t.g.stack[t.g.stack.len() - 2];
    let name = if ability { "Stifle" } else { "Counterspell" };
    cast_new(t, P1, name, &[obj(target)]);
    t.resolve();
    assert!(!t.g.stack.contains(&target));
}

#[test]
fn cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a", "603.4");
    ruling!(
        "Brigone, Soldier of Meletis",
        "Brigone's second ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Gnarlback Rhino",
        "Gnarlback Rhino's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Thunderous Snapper",
        "Thunderous Snapper's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Brigone, Soldier of Meletis");
    supported("Gnarlback Rhino");
    supported("Thunderous Snapper");
    supported("Craw Wurm");
    // Brigone: "Heroic — Whenever you cast a spell that targets Brigone, put a +1/+1
    // counter on Brigone."
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brigone, Soldier of Meletis");
    cast_new(&mut t, P0, "Giant Growth", &[obj(b)]);
    t.settle();
    trigger_on_top(&t, "+1/+1 counter");
    counter_below_top(&mut t, false);
    t.resolve_all();
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    assert_eq!(t.pt(b), (3, 3));
    // Gnarlback Rhino: "Whenever you cast a spell that targets this creature, draw a
    // card."
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Gnarlback Rhino");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Giant Growth", &[obj(r)]);
    t.settle();
    trigger_on_top(&t, "draw a card");
    counter_below_top(&mut t, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(r), (4, 4));
    // Thunderous Snapper: "Whenever you cast a spell with mana value 5 or greater, draw a
    // card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thunderous Snapper");
    let hand = t.hand_size(P0);
    let wurm = cast_new(&mut t, P0, "Craw Wurm", &[]);
    t.settle();
    trigger_on_top(&t, "draw a card");
    counter_below_top(&mut t, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(wurm), Zone::Graveyard(P0));
}

#[test]
fn target_triggers_resolve_before_the_spell() {
    cr!("603.3", "405.5", "608.2");
    ruling!(
        "Wild Defiance",
        "Wild Defiance’s ability will resolve before that instant or sorcery spell."
    );
    ruling!(
        "Angelic Protector",
        "The ability triggers when Angelic Protector is chosen as a target for a spell or ability. The Protectors ability will be put on the stack above that spell or ability, and so resolve first."
    );
    ruling!(
        "Task Force",
        "The +0/+3 bonus is added as a triggered ability upon the casting/activating of a spell or ability which targets this card."
    );
    ruling!(
        "Orvar, the All-Form",
        "Orvar’s first triggered ability resolves (if it resolves; see below) before the spell or ability that caused it to trigger."
    );
    supported("Wild Defiance");
    supported("Angelic Protector");
    supported("Task Force");
    supported("Orvar, the All-Form");
    // Wild Defiance: "Whenever a creature you control becomes the target of an instant or
    // sorcery spell, that creature gets +3/+3 until end of turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wild Defiance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    trigger_on_top(&t, "+3/+3");
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 8));
    // Angelic Protector (2/2) and Task Force (1/3): "Whenever this creature becomes the
    // target of a spell or ability, this creature gets +0/+3 until end of turn." Both
    // survive Lightning Bolt; Task Force's triggers when an ability targets it too.
    for name in ["Angelic Protector", "Task Force"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let (p, tough) = t.pt(c);
        cast_new(&mut t, P1, "Lightning Bolt", &[obj(c)]);
        t.settle();
        trigger_on_top(&t, "+0/+3");
        t.resolve();
        assert_eq!(t.pt(c), (p, tough + 3), "{name}");
        t.resolve_all();
        assert!(t.on_battlefield(c), "{name}");
        let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
        t.activate(P1, sorcerer, 0, &[obj(c)]).expect("ping");
        t.settle();
        trigger_on_top(&t, "+0/+3");
        t.resolve_all();
        assert_eq!(t.pt(c), (p, tough + 6), "{name}");
    }
    // Orvar: the token copy is created while Giant Growth is still on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    trigger_on_top(&t, "token");
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn professor_hojo_draws_even_if_the_ability_is_countered() {
    cr!("603.3", "701.6b", "602.2");
    ruling!(
        "Professor Hojo",
        "Professor Hojo's last ability resolves before the activated ability that caused it to trigger. It resolves even if that activated ability is countered or otherwise leaves the stack."
    );
    supported("Professor Hojo");
    supported("Stifle");
    // "Whenever one or more creatures you control become the target of an activated
    // ability, draw a card. This ability triggers only once each turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Professor Hojo");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let hand = t.hand_size(P0);
    t.activate(P0, sorcerer, 0, &[obj(bears)]).expect("ping");
    t.settle();
    trigger_on_top(&t, "draw a card");
    counter_below_top(&mut t, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn a_spell_targeting_the_creature_several_times_triggers_once() {
    cr!("603.2", "115.3", "601.2c");
    ruling!(
        "Gnarlback Rhino",
        "Gnarlback Rhino's last ability triggers when you cast a spell that has multiple targets as long as at least one of those targets is Gnarlback Rhino. It doesn't trigger multiple times if you cast a spell that targets Gnarlback Rhino multiple times."
    );
    ruling!(
        "Hero of the Games",
        "If you cast a spell that targets this creature multiple times, this ability triggers just once."
    );
    ruling!(
        "Hero of the Nyxborn",
        "If you cast a spell that targets this creature multiple times, this ability triggers just once."
    );
    ruling!(
        "Hero of the Pride",
        "If you cast a spell that targets this creature multiple times, this ability triggers just once."
    );
    ruling!(
        "Hero of the Winds",
        "If you cast a spell that targets this creature multiple times, this ability triggers just once."
    );
    ruling!(
        "Heroes of the Revel",
        "If you cast a spell that targets this creature multiple times, this ability triggers just once."
    );
    ruling!(
        "Wild Defiance",
        "If an instant or sorcery spell targets the same creature you control more than once, Wild Defiance will trigger only once."
    );
    supported("Seeds of Strength");
    // Seeds of Strength: "Target creature gets +1/+1 until end of turn. Target creature
    // gets +1/+1 until end of turn. Target creature gets +1/+1 until end of turn."
    // Gnarlback Rhino: Rhino and two Bears; then the Rhino three times.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Gnarlback Rhino");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_new(
        &mut t,
        P0,
        "Seeds of Strength",
        &[obj(bears), obj(r), obj(bears)],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    cast_new(&mut t, P0, "Seeds of Strength", &[obj(r), obj(r), obj(r)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // The heroes: "Whenever you cast a spell that targets this creature, creatures you
    // control get +1/+0 until end of turn."
    for hero in [
        "Hero of the Games",
        "Hero of the Nyxborn",
        "Hero of the Pride",
        "Hero of the Winds",
        "Heroes of the Revel",
    ] {
        supported(hero);
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, hero);
        let (p, tough) = t.pt(h);
        cast_new(&mut t, P0, "Seeds of Strength", &[obj(h), obj(h), obj(h)]);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "+1/+0"), 1, "{hero}");
        t.resolve_all();
        assert_eq!(t.pt(h), (p + 4, tough + 3), "{hero}");
    }
    // Wild Defiance: one +3/+3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wild Defiance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(
        &mut t,
        P0,
        "Seeds of Strength",
        &[obj(bears), obj(bears), obj(bears)],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "+3/+3"), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 8));
}

#[test]
fn hero_of_leina_tower_pays_x_as_the_ability_resolves() {
    cr!("603.3", "608.2", "107.3");
    ruling!(
        "Hero of Leina Tower",
        "The amount to pay for {X} is chosen as the triggered ability resolves. No player may take actions between the time you pay {X} and the time Hero of Leina Tower has X +1/+1 counters on it."
    );
    supported("Hero of Leina Tower");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hero of Leina Tower");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Giant Growth", &[obj(h)]);
    t.settle();
    let x_asked = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .position(|(_, d)| {
                matches!(d, Decision::ChooseX { .. } | Decision::ChooseNumber { .. })
            })
            .map(|i| i + from)
    };
    // Nothing about X was asked as the spell was cast and the ability triggered.
    assert_eq!(x_asked(&t, from), None);
    colorless(&mut t, P0, 3);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    let before = t.asked().len();
    t.resolve();
    let at = x_asked(&t, before).expect("X chosen on resolution");
    assert_eq!(t.counters(h, counters::PLUS1), 3);
    // No priority between paying and the counters.
    assert!(!t.asked()[at..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn anthousas_lands_have_summoning_sickness() {
    cr!("302.6", "602.5a", "508.1a");
    ruling!(
        "Anthousa, Setessan Hero",
        "If a land that hasn't been under your control continuously since your last turn began becomes a creature, it can't attack and its activated abilities that include {T} can't be activated. Notably, it can't be tapped for mana that turn."
    );
    supported("Anthousa, Setessan Hero");
    // "Heroic — Whenever you cast a spell that targets Anthousa, up to three target lands
    // you control each become 2/2 Warrior creatures until end of turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Anthousa, Setessan Hero");
    let old = t.battlefield(P0, "Plains");
    let new = t.enter(P0, "Forest");
    add_mana(&mut t, P0, ManaType::G, 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(a).go();
    assert!(!tapped(&t, old) && !tapped(&t, new));
    t.answer_targets(P0, &[obj(old), obj(new)]);
    t.resolve_all();
    assert!(t.obj_now(new).is(CardType::Creature));
    assert!(t.obj_now(old).is(CardType::Creature));
    assert!(t.activate(P0, new, 0, &[]).is_err());
    assert!(!tapped(&t, new));
    t.set_step(P0, Step::DeclareAttackers);
    assert!(!crate::r_s02_common::can_attack(&mut t, new));
    assert!(crate::r_s02_common::can_attack(&mut t, old));
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.activate(P0, old, 0, &[]).is_ok());
}

/// P0 casts Giant Growth targeting `conscriptor` with Akroan Conscriptor's heroic ability
/// ("Whenever you cast a spell that targets this creature, gain control of another target
/// creature until end of turn. Untap that creature. It gains haste until end of turn.")
/// targeting `steal`.
fn conscript(t: &mut TestGame, conscriptor: ObjectId, steal: ObjectId) {
    cast_new(t, P0, "Giant Growth", &[obj(conscriptor)]);
    t.answer_targets(P0, &[obj(steal)]);
    t.resolve_all();
}

#[test]
fn akroan_conscriptor_targets_and_attachments() {
    cr!("115.1", "301.5c", "303.4e", "702.103f", "613.1b");
    ruling!(
        "Akroan Conscriptor",
        "The ability can target any creature except Akroan Conscriptor, including one that’s untapped or one you already control."
    );
    ruling!(
        "Akroan Conscriptor",
        "Gaining control of a creature doesn’t cause you to gain control of any Auras or Equipment attached to it. Notably, if that creature is enchanted by an Aura with bestow and dies while under your control, the Aura’s controller will continue to control the creature that Aura becomes."
    );
    supported("Akroan Conscriptor");
    supported("Nyxborn Rollicker");
    // The candidates for its target.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Akroan Conscriptor");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    conscript(&mut t, c, theirs);
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    // The heroic target: P0's own (untapped) creature and P1's, not the Conscriptor.
    assert!(cands
        .iter()
        .any(|v| v.contains(&obj(mine)) && v.contains(&obj(theirs)) && !v.contains(&obj(c))));
    assert_eq!(t.obj_now(theirs).controller, P0);
    // Auras and Equipment stay with their controller; the bestowed Aura becomes a creature
    // under its controller's control.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let c = t.battlefield(P0, "Akroan Conscriptor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 2);
    let rollicker = t.hand(P1, "Nyxborn Rollicker");
    t.cast(P1, rollicker)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve_all();
    let split = crate::r_s06_common::attach_new(&mut t, P1, "Bonesplitter", bears);
    conscript(&mut t, c, bears);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(rollicker).controller, P1);
    assert_eq!(t.obj_now(split).controller, P1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.obj_now(rollicker).is(CardType::Creature));
    assert_eq!(t.obj_now(rollicker).controller, P1);
}

#[test]
fn akroan_conscriptor_and_the_legend_rule() {
    cr!("704.5j", "613.1b");
    ruling!(
        "Akroan Conscriptor",
        "If you control a legendary creature and gain control of another legendary creature with the same name, you’ll choose one to remain on the battlefield and put the other into its owner’s graveyard."
    );
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Akroan Conscriptor");
    let mine = t.battlefield(P0, "Isamaru, Hound of Konda");
    let theirs = t.battlefield(P1, "Isamaru, Hound of Konda");
    t.answer_choose(P0, &[obj(mine)]);
    conscript(&mut t, c, theirs);
    assert_eq!(
        t.named_on_battlefield("Isamaru, Hound of Konda"),
        vec![mine]
    );
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
}

#[test]
fn king_of_the_oathbreakers_triggers_for_each_spirit_targeted() {
    cr!("603.2", "702.26a", "702.26b");
    ruling!(
        "King of the Oathbreakers",
        "If a spell an opponent controls targets multiple Spirits you control, King of the Oathbreakers's triggered ability will trigger once for each of those Spirits."
    );
    supported("King of the Oathbreakers");
    supported("Spectral Sailor");
    supported("Twin Bolt");
    let mut t = TestGame::new(2);
    let king = t.battlefield(P0, "King of the Oathbreakers");
    let sailor = t.battlefield(P0, "Spectral Sailor");
    crate::r_s25_common::lands_for_cost(&mut t, P1, "Twin Bolt");
    let bolt = t.hand(P1, "Twin Bolt");
    t.cast(P1, bolt).targets(&[obj(king), obj(sailor)]).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "phases out"), 2);
    t.resolve_all();
    assert!(t.obj_now(king).phased_out);
    assert!(t.obj_now(sailor).phased_out);
    assert_eq!(t.obj_now(sailor).damage, 0);
}

#[test]
fn for_as_long_as_durations_ignore_phased_out_objects() {
    cr!("702.26d", "611.2b");
    ruling!(
        "King of the Oathbreakers",
        "Any continuous effects with a \"for as long as\" duration, such as that of Rangers of Ithilien, ignore phased-out objects. If ignoring those objects causes the effect's conditions to no longer be met, the duration will expire."
    );
    supported("Kyodai, Soul of Kamigawa");
    // Kyodai (a Spirit): "When Kyodai enters, another target permanent gains
    // indestructible for as long as you control Kyodai." It phases out (King's trigger):
    // the duration expires, even after Kyodai phases back in.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "King of the Oathbreakers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let kyodai = t.enter(P0, "Kyodai, Soul of Kamigawa");
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    cast_new(&mut t, P1, "Shock", &[obj(kyodai)]);
    t.resolve_all();
    assert!(t.obj_now(kyodai).phased_out);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    mtg_engine::keyword_impls::phase_in(&mut t.g, kyodai);
    t.g.recompute();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    destroy(&mut t, bears);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn orvar_copies_one_of_your_targeted_permanents() {
    cr!("603.4", "707.2", "115.1");
    ruling!(
        "Orvar, the All-Form",
        "For the first triggered ability, the instant or sorcery spell can target other things if it also targets one or more permanents you control; however, the token you create can copy only permanents you control that are targets of the spell. If the spell targets more than one permanent you control, you choose which permanent the token is copying as you create the token."
    );
    ruling!(
        "Orvar, the All-Form",
        "For the second triggered ability, you can choose any permanent as the target, regardless of who controls it."
    );
    ruling!(
        "Orvar, the All-Form",
        "If a spell or ability an opponent controls allows or instructs you to choose a card to discard, and you discard Orvar, its last ability will trigger."
    );
    supported("Mind Rot");
    // Seeds of Strength targeting P1's Hill Giant, P0's Bears and P0's Isamaru: P0 picks
    // Isamaru; the Giant can't be copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.answer_choose(P0, &[obj(isamaru)]);
    let from = t.asked().len();
    cast_new(
        &mut t,
        P0,
        "Seeds of Strength",
        &[obj(giant), obj(bears), obj(isamaru)],
    );
    t.resolve_all();
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(offered
        .iter()
        .any(|c| c.contains(&obj(bears)) && c.contains(&obj(isamaru)) && !c.contains(&obj(giant))));
    // Isamaru copied (and the legend rule kept one of them).
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    // Discarding Orvar to P1's Mind Rot ("Target player discards two cards."): a token
    // copy of any target permanent, here P1's Hill Giant.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    let orvar = t.hand(P0, "Orvar, the All-Form");
    t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(orvar)]);
    t.answer_targets(P0, &[obj(giant)]);
    cast_new(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.zone(orvar), Zone::Graveyard(P0));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
    assert!(t
        .named_on_battlefield("Hill Giant")
        .iter()
        .any(|g| t.g.obj(*g).controller == P0));
}

#[test]
fn orvar_checks_the_spells_targets_as_its_ability_resolves() {
    cr!("603.4", "608.2b", "702.16b");
    ruling!(
        "Orvar, the All-Form",
        "As the first triggered ability tries to resolve, the spell that caused it to trigger and that spell’s targets are checked again. Ignore any target of the spell that has left the battlefield by that point. If all of the permanents you controlled that were targets have left the battlefield by that point, the triggered ability will do nothing and no token will be created. If at least one permanent you control that’s a target is still on the battlefield, the triggered ability will resolve, even if that permanent is now an illegal target for the spell."
    );
    ruling!(
        "Orvar, the All-Form",
        "If the spell itself has left the stack by that point, use the targets it had when it left the stack to perform the check described above."
    );
    supported("Gods Willing");
    // The target left the battlefield: no token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    // The target became illegal (protection from green from Gods Willing, which itself
    // triggers Orvar): both abilities make a token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    let green = Color::ALL.iter().position(|c| *c == Color::Green).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(green));
    cast_new(&mut t, P0, "Gods Willing", &[obj(bears)]);
    t.resolve(); // Orvar's trigger for Gods Willing
    t.resolve(); // Gods Willing: protection from green
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    t.resolve(); // Orvar's trigger for Giant Growth
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 3);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    // The spell was countered before the ability resolved: its targets as it left.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    counter_below_top(&mut t, false);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn orvars_token_copies_only_copiable_values() {
    cr!("707.2", "707.3", "707.4", "202.3e");
    ruling!(
        "Orvar, the All-Form",
        "The token copies exactly what is printed on the permanent and nothing else (unless that permanent is a token or is copying something else; see below). It doesn’t copy whether that permanent is tapped or untapped, whether it has any counters on it, whether it has any Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Orvar, the All-Form",
        "If the permanent being copied has {X} in its mana cost, X is 0."
    );
    ruling!(
        "Orvar, the All-Form",
        "If the permanent being copied is copying something else, the token you create will use the copiable values of the that permanent. In most cases, it will be a copy of whatever the permanent is copying. If it’s copying a permanent or card with {X} in its mana cost, X is 0."
    );
    ruling!(
        "Orvar, the All-Form",
        "If the permanent being copied is itself a token, the token created by the last ability copies the original characteristics of that token as stated by the effect that created it."
    );
    supported("Chalice of the Void");
    supported("Clone");
    supported("Twiddle");
    let new_token = |t: &TestGame, before: &[ObjectId]| -> ObjectId {
        *t.g.battlefield
            .iter()
            .find(|id| !before.contains(id) && t.g.obj(**id).is_token())
            .expect("a new token")
    };
    // Tapped, with counters, an Aura, and a pump: the token has none of it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    t.g.add_counters(obj(bears), counters::PLUS1, 2, None);
    crate::r_s06_common::attach_new(&mut t, P0, "Rancor", bears);
    let before = t.g.battlefield.clone();
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve();
    let tok = new_token(&t, &before);
    assert!(!tapped(&t, tok));
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    assert_eq!(t.pt(tok), (2, 2));
    assert!(t
        .g
        .battlefield
        .iter()
        .all(|a| t.g.obj(*a).attached_to != Some(obj(tok))));
    t.resolve_all();
    // Chalice of the Void ({X}{X}) with X=2: the token has mana value 0 and no counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let chalice = t.hand(P0, "Chalice of the Void");
    colorless(&mut t, P0, 4);
    t.cast(P0, chalice).x(2).go();
    t.resolve_all();
    assert_eq!(t.counters(chalice, "charge"), 2);
    let chalice = t.g.current(chalice);
    let before = t.g.battlefield.clone();
    // Twiddle: "You may tap or untap target artifact, creature, or land."
    cast_new(&mut t, P0, "Twiddle", &[obj(chalice)]);
    t.resolve_all();
    let tok = new_token(&t, &before);
    assert_eq!(t.obj_now(tok).chars.name.as_str(), "Chalice of the Void");
    assert_eq!(t.obj_now(tok).chars.mana_value(), 0);
    assert_eq!(t.counters(tok, "charge"), 0);
    // A Clone copying Grizzly Bears: the token is a Grizzly Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    let before = t.g.battlefield.clone();
    cast_new(&mut t, P0, "Giant Growth", &[obj(clone)]);
    t.resolve_all();
    let tok = new_token(&t, &before);
    assert_eq!(t.obj_now(tok).chars.name.as_str(), "Grizzly Bears");
    // A token (a 1/1 Soldier from Raise the Alarm): the copy is the token as created.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    t.g.add_counters(obj(soldier), counters::PLUS1, 1, None);
    let before = t.g.battlefield.clone();
    cast_new(&mut t, P0, "Giant Growth", &[obj(soldier)]);
    t.resolve_all();
    let tok = new_token(&t, &before);
    assert!(t.obj_now(tok).chars.has_subtype("Soldier"));
    assert_eq!(t.pt(tok), (1, 1));
}

#[test]
fn battlefield_thaumaturge_reduces_the_total_cost() {
    cr!("601.2f", "118.7a", "202.3", "118.8d");
    ruling!(
        "Battlefield Thaumaturge",
        "Battlefield Thaumaturge's first ability doesn't change that spell's mana cost or mana value. It changes only the amount you actually pay (the “total cost” of the spell)."
    );
    ruling!(
        "Battlefield Thaumaturge",
        "To calculate the total cost of an instant or sorcery spell you're casting, start with the mana cost (or alternative cost, if applicable), add all cost increases (including additional costs such as those imposed by strive abilities), then subtract all cost reductions (such as the one created by Battlefield Thaumaturge's first ability)."
    );
    supported("Battlefield Thaumaturge");
    supported("Launch the Fleet");
    supported("Murder");
    // Murder ({1}{B}{B}) targeting a creature: paid with {B}{B}; its mana value is 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Battlefield Thaumaturge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::B, 2);
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(bears).go();
    assert_eq!(t.g.obj(t.g.current(murder)).chars.mana_value(), 3);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // Launch the Fleet ({W}, strive: {1} more for each target beyond the first) with
    // three targets: {2}{W}, minus {3}: {W}.
    let mut t = TestGame::new(2);
    let th = t.battlefield(P0, "Battlefield Thaumaturge");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    add_mana(&mut t, P0, ManaType::W, 1);
    let fleet = t.hand(P0, "Launch the Fleet");
    t.cast(P0, fleet).targets(&[obj(th), obj(a), obj(b)]).go();
    assert_eq!(t.zone(fleet), Zone::Stack);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
}

#[test]
fn battlefield_thaumaturge_counts_each_target_creature() {
    cr!("601.2f", "115.3");
    interp!("thaumaturge-counts-each-time-a-creature-is-targeted");
    ruling!(
        "Battlefield Thaumaturge",
        "The cost reduction is {1} per target creature, even if the spell targets a creature more than once."
    );
    supported("Epic Fight");
    supported("Thalia, Guardian of Thraben");
    // Epic Fight ({2}{G}, "Choose one or both — • Double target creature's power and
    // toughness until end of turn. • Target creature you control fights target creature an
    // opponent controls.") plus {1} from Thalia: {3}{G}. Both modes, the Bears targeted
    // twice and the Giant once: three target creatures, {3} less: {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Battlefield Thaumaturge");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::G, 1);
    let fight = t.hand(P0, "Epic Fight");
    t.cast(P0, fight)
        .modes(&[0, 1])
        .target(bears)
        .target(bears)
        .target(giant)
        .go();
    assert_eq!(t.zone(fight), Zone::Stack);
    t.resolve_all();
    // Doubled to 4/4, it fights the 3/3 Giant.
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(bears));
}
