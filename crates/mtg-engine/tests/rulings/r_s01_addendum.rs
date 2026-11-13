//! Rulings batch S01 — addendum (an ability word, CR 207.2c): "If you cast this spell
//! during your main phase, [effect]."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts Sphinx's Insight ("Draw two cards. Addendum — If you cast this spell during your
/// main phase, you gain 2 life.") for `p` in the current step.
fn cast_insight(t: &mut TestGame, p: PlayerId) -> ObjectId {
    give_mana_for(t, p, "Sphinx's Insight");
    let c = t.hand(p, "Sphinx's Insight");
    t.cast(p, c).go()
}

#[test]
fn addendum_applies_only_to_a_spell_cast_during_your_main_phase() {
    cr!("207.2c", "505.1");
    supported("Sphinx's Insight");
    supported("Arrester's Admonition");
    // During your main phase: the bonus applies.
    let mut t = TestGame::new(2);
    cast_insight(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 22);
    // During your combat phase: it doesn't.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    cast_insight(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 20);
    // During an opponent's main phase: it doesn't either ("your main phase").
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Arrester's Admonition");
    // "Return target creature to its owner's hand. Addendum — ... draw a card."
    let c = t.hand(P0, "Arrester's Admonition");
    t.cast(P0, c).target(bears).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn a_copy_of_an_addendum_spell_wasnt_cast_and_gets_no_bonus() {
    cr!("707.10");
    ruling!(
        "Sphinx's Insight",
        "If an effect copies a spell with an addendum ability while it's on the stack, the copy wasn't cast at all, so you won't get the addendum bonus."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    let insight = cast_insight(&mut t, P0);
    t.lands(P0, "Island", 2);
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(insight).go();
    // Twincast, then the copy: two cards, no life.
    t.resolve();
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 20);
    // The original: two more cards and 2 life.
    t.resolve();
    assert_eq!(t.hand_size(P0), 4);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn addendum_applies_as_the_spell_resolves_and_not_if_it_is_countered() {
    cr!("608.2c", "701.6a");
    ruling!(
        "Sphinx's Insight",
        "Addendum abilities of instant spells apply while the spell is resolving, not immediately after casting it. If the spell is countered, you don't get the addendum bonus."
    );
    supported("Counterspell");
    let mut t = TestGame::new(2);
    let insight = cast_insight(&mut t, P0);
    // Nothing happened on casting.
    assert_eq!(t.life(P0), 20);
    t.lands(P1, "Island", 2);
    let counter = t.hand(P1, "Counterspell");
    t.cast(P1, counter).target(insight).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sphinx's Insight"));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn an_addendum_paragraph_refers_to_the_creature_the_spell_targeted() {
    cr!("113.3a", "207.2c", "608.2c");
    supported("Arrester's Zeal");
    // "Target creature gets +2/+2 until end of turn. Addendum — If you cast this spell
    // during your main phase, that creature gains flying until end of turn."
    for main_phase in [true, false] {
        let mut t = TestGame::new(2);
        if !main_phase {
            t.set_step(P0, Step::BeginningOfCombat);
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        give_mana_for(&mut t, P0, "Arrester's Zeal");
        let c = t.hand(P0, "Arrester's Zeal");
        t.cast(P0, c).target(bears).go();
        t.resolve_all();
        assert_eq!(t.pt(bears), (4, 4));
        assert_eq!(
            t.obj(bears).chars.has_keyword(KeywordKind::Flying),
            main_phase
        );
    }
}

#[test]
fn code_of_constraint_keeps_an_already_tapped_creature_tapped() {
    cr!("502.3", "608.2c");
    ruling!(
        "Code of Constraint",
        "Code of Constraint can target a creature that's already tapped. If you cast it during your main phase, that creature won't untap during its controller's next untap step."
    );
    supported("Code of Constraint");
    // "Target creature gets -4/-0 until end of turn. Draw a card. Addendum — If you cast
    // this spell during your main phase, tap that creature and it doesn't untap during its
    // controller's next untap step."
    for main_phase in [true, false] {
        let mut t = TestGame::new(2);
        if !main_phase {
            t.set_step(P0, Step::End);
        }
        let ogre = t.battlefield(P1, "Gray Ogre");
        let other = t.battlefield(P1, "Grizzly Bears");
        t.g.tap(ogre);
        t.g.tap(other);
        give_mana_for(&mut t, P0, "Code of Constraint");
        let c = t.hand(P0, "Code of Constraint");
        t.cast(P0, c).target(ogre).go();
        t.resolve_all();
        assert_eq!(t.pt(ogre), (-2, 2));
        assert_eq!(t.hand_size(P0), 1);
        t.advance_to(P1, Step::Upkeep);
        assert!(!t.obj(other).tapped);
        assert_eq!(t.obj(ogre).tapped, main_phase);
    }
}

#[test]
fn an_addendum_aura_that_enters_without_being_cast_doesnt_trigger() {
    cr!("603.4", "505.1");
    ruling!(
        "Sentinel's Mark",
        "If Sentinel's Mark enters the battlefield without being cast, the addendum ability won't trigger, even if it's your main phase."
    );
    supported("Sentinel's Mark");
    // "Enchanted creature gets +1/+2 and has vigilance. Addendum — When this Aura enters,
    // if you cast it during your main phase, enchanted creature gains lifelink until end of
    // turn."
    let lifelink = |t: &TestGame, c: ObjectId| t.obj(c).chars.has_keyword(KeywordKind::Lifelink);
    // Cast during your main phase, and during combat.
    for main_phase in [true, false] {
        let mut t = TestGame::new(2);
        if !main_phase {
            t.set_step(P0, Step::BeginningOfCombat);
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        give_mana_for(&mut t, P0, "Sentinel's Mark");
        let c = t.hand(P0, "Sentinel's Mark");
        t.cast(P0, c).target(bears).go();
        t.resolve();
        assert_eq!(t.stack_len(), usize::from(main_phase));
        t.resolve_all();
        assert_eq!(t.pt(bears), (3, 4));
        assert_eq!(lifelink(&t, bears), main_phase);
    }
    // Put onto the battlefield during your main phase without being cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let mark = t.enter(P0, "Sentinel's Mark");
    t.settle();
    assert_eq!(t.obj(mark).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(bears), (3, 4));
    assert!(!lifelink(&t, bears));
}

#[test]
fn unbreakable_formation_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c", "608.2c");
    ruling!(
        "Unbreakable Formation",
        "Unbreakable Formation affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't gain indestructible or vigilance and they won't get a +1/+1 counter."
    );
    supported("Unbreakable Formation");
    // "Creatures you control gain indestructible until end of turn. Addendum — If you
    // cast this spell during your main phase, put a +1/+1 counter on each of those
    // creatures and they gain vigilance until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, "Unbreakable Formation");
    let c = t.hand(P0, "Unbreakable Formation");
    t.cast(P0, c).go();
    t.resolve_all();
    let later = t.battlefield(P0, "Gray Ogre");
    t.g.recompute();
    let has = |t: &TestGame, c: ObjectId, k: KeywordKind| t.obj(c).chars.has_keyword(k);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(has(&t, bears, KeywordKind::Indestructible));
    assert!(has(&t, bears, KeywordKind::Vigilance));
    for c in [later, theirs] {
        assert_eq!(t.counters(c, "+1/+1"), 0);
        assert!(!has(&t, c, KeywordKind::Indestructible));
        assert!(!has(&t, c, KeywordKind::Vigilance));
    }
}
