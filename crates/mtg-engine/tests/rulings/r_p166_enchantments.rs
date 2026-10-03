//! Rulings batch P166 — enchantment synergies: artifact enchantments counted once, cast
//! triggers resolving first, "put into a graveyard from the battlefield" triggers (Roles,
//! Auras falling off), intervening "if you control an enchantment" clauses (CR 603.4),
//! and costs and choices involving enchantments.

use crate::r_p125_common::at_p1;
use crate::r_p146_common::{counter, destroy_together};
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy, target_candidates};
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::{move_to, run_from};
use crate::r_s06_common::*;
use mtg_engine::ability::{Effect, PlayerRef, Sel, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Artifact enchantments
// ---------------------------------------------------------------------------------------

#[test]
fn runaway_trash_bot_counts_an_artifact_enchantment_card_once() {
    cr!("205.2a", "604.3");
    ruling!(
        "Runaway Trash-Bot",
        "A card in your graveyard that is both an artifact and an enchantment is only counted once."
    );
    supported("Runaway Trash-Bot");
    let mut t = TestGame::new(2);
    let bot = t.battlefield(P0, "Runaway Trash-Bot");
    t.graveyard(P0, "Bow of Nylea");
    t.settle();
    assert_eq!(t.pt(bot), (1, 4));
    t.graveyard(P0, "Ornithopter");
    t.graveyard(P0, "Pacifism");
    t.settle();
    assert_eq!(t.pt(bot), (3, 4));
}

#[test]
fn shinechaser_gets_both_bonuses_from_an_artifact_enchantment() {
    cr!("205.2a", "613.4c");
    ruling!(
        "Shinechaser",
        "A permanent that's both an artifact and an enchantment satisfies both of Shinechaser's abilities, so Shinechaser gets +2/+2."
    );
    supported("Shinechaser");
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Shinechaser");
    assert_eq!(t.pt(sc), (1, 1));
    t.battlefield(P0, "Bow of Nylea");
    t.settle();
    assert_eq!(t.pt(sc), (3, 3));
}

/// P0 casts Bow of Nylea (an artifact and enchantment spell) with Alela out; returns the
/// game with the spell on the stack (triggers stacked).
fn alela_bow() -> (TestGame, ObjectId) {
    supported("Alela, Artful Provocateur");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alela, Artful Provocateur");
    let bow = in_hand_with_mana(&mut t, P0, "Bow of Nylea");
    let spell = t.cast(P0, bow).go();
    t.settle();
    (t, spell)
}

fn faeries(t: &TestGame) -> usize {
    t.named_on_battlefield("Faerie Token").len()
}

#[test]
fn alela_triggers_once_for_an_artifact_enchantment_spell() {
    cr!("603.2", "205.2a");
    ruling!(
        "Alela, Artful Provocateur",
        "A spell that's both an artifact and an enchantment spell causes Alela's last ability to trigger only once."
    );
    let (mut t, _) = alela_bow();
    assert_eq!(triggers_on_stack(&t, "Faerie"), 1);
    t.resolve_all();
    assert_eq!(faeries(&t), 1);
}

#[test]
fn alela_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Alela, Artful Provocateur",
        "Alela's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let (mut t, spell) = alela_bow();
    assert_eq!(top_of_stack(&t) != spell, true);
    t.resolve();
    assert_eq!(faeries(&t), 1);
    assert!(t.g.stack.contains(&spell));
    // And when the spell is countered.
    let (mut t, spell) = alela_bow();
    counter(&mut t, spell);
    t.resolve_all();
    assert_eq!(faeries(&t), 1);
}

#[test]
fn cast_enchantment_triggers_resolve_before_the_spell() {
    cr!("603.3", "601.2i");
    ruling!(
        "Blessed Spirits",
        "Blessed Spirits’s ability will resolve before the enchantment spell that caused it to trigger."
    );
    ruling!(
        "Blightcaster",
        "Blightcaster's ability will resolve before the enchantment spell that caused it to trigger."
    );
    supported("Blessed Spirits");
    supported("Blightcaster");
    let mut t = TestGame::new(2);
    let spirits = t.battlefield(P0, "Blessed Spirits");
    t.battlefield(P0, "Blightcaster");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = in_hand_with_mana(&mut t, P0, "Holy Strength");
    t.answer_targets(P0, &[Entity::Object(spirits)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    let spell = t.cast(P0, card).go();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve();
    t.resolve();
    // Both triggers resolved; the Aura is still on the stack.
    assert!(t.g.stack.contains(&spell));
    assert_eq!(t.counters(spirits, counters::PLUS1), 1);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn sigil_of_the_empty_throne_doesnt_see_itself_cast() {
    cr!("603.2", "113.6", "601.2i");
    ruling!(
        "Sigil of the Empty Throne",
        "Because it's not on the battlefield yet, casting Sigil of the Empty Throne won't cause its own ability to trigger."
    );
    supported("Sigil of the Empty Throne");
    let mut t = TestGame::new(2);
    let card = in_hand_with_mana(&mut t, P0, "Sigil of the Empty Throne");
    t.cast(P0, card).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.named_on_battlefield("Angel Token").is_empty());
    // The next enchantment spell triggers it.
    let card = in_hand_with_mana(&mut t, P0, "Pacifism");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.cast(P0, card).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Angel Token").len(), 1);
}

#[test]
fn blightcaster_must_target_your_own_creature_but_may_not_shrink_it() {
    cr!("603.3d", "115.1", "608.2d");
    ruling!(
        "Blightcaster",
        "If you are the only player who controls a creature when Blightcaster's ability triggers, you must choose one as the target, although you can choose not to give it -2/-2."
    );
    ruling!(
        "Blightcaster",
        "If you are the only player who controls a creature when Blightcaster's ability triggers, you must choose one of those creatures as the target, although you can choose to not give it -2/-2."
    );
    let mut t = TestGame::new(2);
    let caster = t.battlefield(P0, "Blightcaster");
    let card = in_hand_with_mana(&mut t, P0, "Sigil of the Empty Throne");
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.cast(P0, card).go();
    t.settle();
    assert_eq!(
        target_candidates(&t, P0, from),
        vec![vec![Entity::Object(caster)]]
    );
    assert_eq!(triggers_on_stack(&t, "-2/-2"), 1);
    t.resolve_all();
    assert_eq!(t.pt(caster), (2, 3));
}

#[test]
fn weaver_of_harmony_copies_a_cycling_ability_of_an_enchantment_card() {
    cr!("707.10", "113.7", "702.29a");
    ruling!(
        "Weaver of Harmony",
        "Activated and triggered abilities from enchantment sources include abilities of enchantment cards that can be activated or triggered from other zones, such as channel abilities."
    );
    supported("Weaver of Harmony");
    supported("Cast Out");
    let mut t = TestGame::new(2);
    let weaver = t.battlefield(P0, "Weaver of Harmony");
    let cast_out = t.hand(P0, "Cast Out");
    stack_library(&mut t, P0, &["Opt", "Hill Giant", "Grizzly Bears"]);
    t.lands(P0, "Plains", 1);
    cycle(&mut t, P0, cast_out, 0).unwrap();
    let ability = top_of_stack(&t);
    t.lands(P0, "Forest", 1);
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Object(ability)]);
    activate_containing(&mut t, P0, weaver, "Copy").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn tenacious_tomeseeker_cant_return_an_adventurer_card() {
    cr!("715.4", "702.166a");
    ruling!(
        "Tenacious Tomeseeker",
        "Adventurer cards aren't instant or sorcery cards while they're in your graveyard. You can't use Tenacious Tomeseeker's ability to return an adventurer card from your graveyard to your hand."
    );
    supported("Tenacious Tomeseeker");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Bonecrusher Giant");
    let opt = t.graveyard(P0, "Opt");
    let thopter = t.battlefield(P0, "Ornithopter");
    let card = in_hand_with_mana(&mut t, P0, "Tenacious Tomeseeker");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.answer_targets(P0, &[Entity::Object(opt)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(!t.on_battlefield(thopter), "bargained");
    let offered = target_candidates(&t, P0, from).concat();
    assert!(offered.contains(&Entity::Object(opt)));
    assert!(!offered.contains(&Entity::Object(giant)));
    assert!(t.in_hand(P0, "Opt"));
}

// ---------------------------------------------------------------------------------------
// Ghen, Arcanum Weaver
// ---------------------------------------------------------------------------------------

#[test]
fn ghen_returns_an_aura_without_targeting() {
    cr!("303.4f", "303.4a");
    ruling!(
        "Ghen, Arcanum Weaver",
        "An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, it remains in your graveyard."
    );
    supported("Ghen, Arcanum Weaver");
    let ghen_setup = |t: &mut TestGame| {
        let ghen = t.battlefield(P0, "Ghen, Arcanum Weaver");
        t.battlefield(P0, "Sigil of the Empty Throne");
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Swamp", 1);
        ghen
    };
    // Pacifism onto P1's hexproof Gladecover Scout.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let pacifism = t.graveyard(P0, "Pacifism");
    let ghen = ghen_setup(&mut t);
    t.answer_targets(P0, &[Entity::Object(pacifism)]);
    t.answer_choose(P0, &[Entity::Object(scout)]);
    activate_containing(&mut t, P0, ghen, "Return").unwrap();
    t.resolve_all();
    let p = t.g.current(pacifism);
    assert!(t.on_battlefield(p));
    assert_eq!(attached_to(&t, p), Some(Entity::Object(scout)));
    // With no creature, Pacifism stays in the graveyard. (Ghen sacrifices itself? No:
    // Ghen is a creature, so make it leave first.)
    let mut t = TestGame::new(2);
    let pacifism = t.graveyard(P0, "Pacifism");
    let ghen = ghen_setup(&mut t);
    t.answer_targets(P0, &[Entity::Object(pacifism)]);
    activate_containing(&mut t, P0, ghen, "Return").unwrap();
    move_to(&mut t, ghen, Zone::Exile);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Pacifism"));
}

#[test]
fn ghen_cant_target_the_enchantment_it_sacrifices() {
    cr!("601.2c", "601.2h", "602.2b");
    ruling!(
        "Ghen, Arcanum Weaver",
        "Because targets are chosen before costs are paid (such as the cost of sacrificing an enchantment), Ghen's ability can't target the enchantment you intend to sacrifice to activate its ability."
    );
    let mut t = TestGame::new(2);
    let ghen = t.battlefield(P0, "Ghen, Arcanum Weaver");
    t.battlefield(P0, "Sigil of the Empty Throne");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    // No enchantment card in the graveyard yet: no legal target, so it can't be activated.
    assert!(!can_activate(&mut t, P0, ghen));
    assert!(activate_containing(&mut t, P0, ghen, "Return").is_err());
    assert!(t.named_on_battlefield("Sigil of the Empty Throne").len() == 1);
}

// ---------------------------------------------------------------------------------------
// Malevolent Witchkite
// ---------------------------------------------------------------------------------------

#[test]
fn witchkite_may_sacrifice_nothing() {
    cr!("107.1c", "701.21a");
    ruling!(
        "Malevolent Witchkite",
        "As Malevolent Witchkite's triggered ability resolves, you may choose to sacrifice zero permanents."
    );
    ruling!(
        "Malevolent Witchkite",
        "Those abilities won't trigger if you choose to sacrifice zero permanents."
    );
    supported("Malevolent Witchkite");
    supported("Knight of Doves");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Knight of Doves");
    let sigil = t.battlefield(P0, "Sigil of the Empty Throne");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[]);
    t.enter(P0, "Malevolent Witchkite");
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(sigil));
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.named_on_battlefield("Bird Token").is_empty());
}

#[test]
fn witchkite_draws_before_sacrifice_triggers_go_on_the_stack() {
    cr!("603.3", "608.2");
    ruling!(
        "Malevolent Witchkite",
        "If any abilities trigger when you sacrifice permanents, those abilities won't be put onto the stack until after you've drawn cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Knight of Doves");
    let sigil = t.battlefield(P0, "Sigil of the Empty Throne");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Object(sigil)]);
    t.enter(P0, "Malevolent Witchkite");
    t.settle();
    t.resolve();
    assert!(!t.on_battlefield(sigil));
    assert_eq!(t.hand_size(P0), hand + 1);
    // The Knight's trigger is put on the stack afterwards.
    assert_eq!(triggers_on_stack(&t, "Bird"), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bird Token").len(), 1);
}

// ---------------------------------------------------------------------------------------
// "Whenever an enchantment you control is put into a graveyard from the battlefield"
// ---------------------------------------------------------------------------------------

#[test]
fn role_tokens_going_to_the_graveyard_trigger() {
    cr!("111.7", "111.8", "603.6c");
    ruling!(
        "Ashiok's Reaper",
        "Enchantment tokens (such as Roles) that are sacrificed, destroyed, or would otherwise go to the graveyard are put into their owner's graveyard before ceasing to exist. If you controlled the token, Ashiok's Reaper's ability will trigger."
    );
    ruling!(
        "Knight of Doves",
        "Enchantment tokens (such as Roles) that are sacrificed, destroyed, or would otherwise go to the graveyard are put into their owner's graveyard before ceasing to exist. If you controlled the token, Knight of Doves's ability will trigger."
    );
    ruling!(
        "Savior of the Sleeping",
        "Enchantment tokens (such as Roles) that are sacrificed, destroyed, or would otherwise go to the graveyard are put into their owner's graveyard before ceasing to exist. If you controlled the token, Savior of the Sleeping's ability will trigger."
    );
    ruling!(
        "Warehouse Tabby",
        "Enchantment tokens (such as Roles) that are sacrificed, destroyed, or would otherwise go to the graveyard are put into their owner's graveyard before ceasing to exist. If you controlled the token, Warehouse Tabby's ability will trigger."
    );
    for name in [
        "Ashiok's Reaper",
        "Knight of Doves",
        "Savior of the Sleeping",
        "Warehouse Tabby",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let watcher = t.battlefield(P0, name);
        let bears = t.battlefield(P1, "Grizzly Bears");
        stack_library(&mut t, P0, &["Opt"]);
        let spec = mtg_engine::tokens::predefined("Cursed").expect("Cursed Role");
        run_from(
            &mut t,
            P0,
            None,
            Effect::CreateTokenAttached {
                spec,
                count: Value::c(1),
                controller: PlayerRef::You,
                to: Sel::Target(0),
            },
            &[Entity::Object(bears)],
        );
        let role = crate::r_s15_common::roles_on(&t, bears)[0];
        let hand = t.hand_size(P0);
        destroy(&mut t, role);
        t.resolve_all();
        let fired = match name {
            "Ashiok's Reaper" => t.hand_size(P0) == hand + 1,
            "Knight of Doves" => t.named_on_battlefield("Bird Token").len() == 1,
            "Savior of the Sleeping" => t.counters(watcher, counters::PLUS1) == 1,
            _ => t.named_on_battlefield("Rat Token").len() == 1,
        };
        assert!(fired, "{name}");
    }
}

#[test]
fn auras_falling_off_a_dead_enchantment_watcher_dont_trigger_it() {
    cr!("603.10a", "704.5m", "603.6c");
    ruling!(
        "Knight of Doves",
        "If Knight of Doves leaves the battlefield while it has an Aura you control attached to it or at the same time as another creature with an Aura you control attached to it, its ability won't trigger for those enchantments unless they were destroyed by the same effect that caused Knight of Doves to leave the battlefield."
    );
    ruling!(
        "Warehouse Tabby",
        "If Warehouse Tabby leaves the battlefield while it has an Aura you control attached to it or at the same time as another creature with an Aura you control attached to it, its ability won't trigger for those enchantments unless they were destroyed by the same effect that caused Warehouse Tabby to leave the battlefield."
    );
    ruling!(
        "Femeref Enchantress",
        "If it goes to the graveyard, its ability will not trigger because of any Auras on herself unless those Auras are being destroyed by the same effect that is destroying the Enchantress. This is because those Auras are not put into the graveyard until after she is in the graveyard."
    );
    supported("Femeref Enchantress");
    supported("Warehouse Tabby");
    // (name, did its ability fire for the Aura)
    let fired = |t: &TestGame, name: &str, hand: usize| match name {
        "Knight of Doves" => !t.named_on_battlefield("Bird Token").is_empty(),
        "Warehouse Tabby" => !t.named_on_battlefield("Rat Token").is_empty(),
        _ => t.hand_size(P0) > hand,
    };
    for name in ["Knight of Doves", "Warehouse Tabby", "Femeref Enchantress"] {
        // Destroyed alone: the Aura goes to the graveyard later, as a state-based action.
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &["Opt", "Opt"]);
        let watcher = t.battlefield(P0, name);
        attach_new(&mut t, P0, "Holy Strength", watcher);
        let hand = t.hand_size(P0);
        destroy(&mut t, watcher);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Holy Strength"), "{name}");
        assert!(!fired(&t, name, hand), "{name}");
        // Destroyed together with the Aura by the same effect: it triggers.
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &["Opt", "Opt"]);
        let watcher = t.battlefield(P0, name);
        let aura = attach_new(&mut t, P0, "Holy Strength", watcher);
        let hand = t.hand_size(P0);
        destroy_together(&mut t, &[watcher, aura]);
        t.resolve_all();
        assert!(fired(&t, name, hand), "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Riptide Chimera
// ---------------------------------------------------------------------------------------

#[test]
fn riptide_chimera_returns_itself_if_its_the_only_enchantment() {
    cr!("608.2c", "303.1");
    ruling!(
        "Riptide Chimera",
        "If Riptide Chimera is the only enchantment you control when its ability resolves, you must return it to its owner's hand."
    );
    supported("Riptide Chimera");
    let mut t = TestGame::new(2);
    let chimera = t.battlefield(P0, "Riptide Chimera");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(!t.on_battlefield(chimera));
    assert!(t.in_hand(P0, "Riptide Chimera"));
}

#[test]
fn riptide_chimera_does_nothing_without_an_enchantment() {
    cr!("608.2c", "609.3");
    ruling!(
        "Riptide Chimera",
        "If you control no enchantments when Riptide Chimera's ability resolves (perhaps because Riptide Chimera stops being an enchantment for some reason), then nothing happens."
    );
    // The Chimera leaves with its trigger on the stack: no enchantment to return.
    let mut t = TestGame::new(2);
    let chimera = t.battlefield(P0, "Riptide Chimera");
    let bears = t.battlefield(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "return an enchantment"), 1);
    move_to(&mut t, chimera, Zone::Exile);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// "If you control an enchantment" (intervening if)
// ---------------------------------------------------------------------------------------

#[test]
fn blood_cursed_knight_counts_your_aura_on_an_opponents_creature() {
    cr!("303.4e", "108.4");
    ruling!(
        "Blood-Cursed Knight",
        "If you cast an Aura spell targeting a creature controlled by an opponent, you still control that Aura. It will count for Blood-Cursed Knight’s ability."
    );
    supported("Blood-Cursed Knight");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Blood-Cursed Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(knight), (3, 2));
    let card = in_hand_with_mana(&mut t, P0, "Pacifism");
    t.cast(P0, card).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert_eq!(t.pt(knight), (4, 3));
    assert!(has_kw(&t, knight, KeywordKind::Lifelink));
}

#[test]
fn lagonna_band_elder_checks_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Lagonna-Band Elder",
        "If you don't control an enchantment when Lagonna-Band Elder enters the battlefield, its ability won't trigger. If it does trigger, but you don't control an enchantment when the ability tries to resolve, you won't gain life."
    );
    supported("Lagonna-Band Elder");
    // No enchantment: no trigger.
    let mut t = TestGame::new(2);
    t.enter(P0, "Lagonna-Band Elder");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // The enchantment leaves before it resolves: no life.
    let mut t = TestGame::new(2);
    let sigil = t.battlefield(P0, "Sigil of the Empty Throne");
    t.enter(P0, "Lagonna-Band Elder");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, sigil, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Otherwise 3 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sigil of the Empty Throne");
    t.enter(P0, "Lagonna-Band Elder");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn kami_of_terrible_secrets_needs_both_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Kami of Terrible Secrets",
        "If you don't control both an artifact and an enchantment at the moment Kami of Terrible Secrets enters the battlefield, its ability won't trigger at all. If the ability triggers, it will check again as it tries to resolve. If you don't control both at that time, the ability won't do anything."
    );
    supported("Kami of Terrible Secrets");
    // Only an enchantment: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sigil of the Empty Throne");
    t.enter(P0, "Kami of Terrible Secrets");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Both, then the artifact leaves: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sigil of the Empty Throne");
    let thopter = t.battlefield(P0, "Ornithopter");
    let hand = t.hand_size(P0);
    t.enter(P0, "Kami of Terrible Secrets");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, thopter, Zone::Exile);
    t.resolve_all();
    assert_eq!((t.life(P0), t.hand_size(P0)), (20, hand));
    // An artifact enchantment is both.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bow of Nylea");
    let hand = t.hand_size(P0);
    t.enter(P0, "Kami of Terrible Secrets");
    t.settle();
    t.resolve_all();
    assert_eq!((t.life(P0), t.hand_size(P0)), (21, hand + 1));
}

fn pilgrim_attack(t: &mut TestGame) -> ObjectId {
    let pilgrim = t.battlefield(P0, "Stone Haven Pilgrim");
    t.set_step(P0, Step::PrecombatMain);
    attack_with(t, &at_p1(&[pilgrim]));
    pilgrim
}

#[test]
fn stone_haven_pilgrim_checks_twice_but_any_artifact_or_enchantment_will_do() {
    cr!("603.4");
    ruling!(
        "Stone Haven Pilgrim",
        "If you don’t control an artifact or enchantment immediately after Stone Haven Pilgrim attacks, its ability doesn’t trigger. If you don’t control one as the ability resolves, it doesn’t get +1/+1 or gain lifelink. It doesn’t have to be the same artifact or enchantment both times, however."
    );
    supported("Stone Haven Pilgrim");
    // Nothing: no trigger.
    let mut t = TestGame::new(2);
    pilgrim_attack(&mut t);
    assert_eq!(t.stack_len(), 0);
    // Gone by resolution: no bonus.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let pilgrim = pilgrim_attack(&mut t);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, thopter, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.pt(pilgrim), (2, 2));
    // A different one at resolution: bonus.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let pilgrim = pilgrim_attack(&mut t);
    move_to(&mut t, thopter, Zone::Exile);
    t.battlefield(P0, "Sigil of the Empty Throne");
    t.resolve_all();
    assert_eq!(t.pt(pilgrim), (3, 3));
    assert!(has_kw(&t, pilgrim, KeywordKind::Lifelink));
}

#[test]
fn stone_haven_pilgrim_gets_the_bonus_once() {
    cr!("603.4", "611.2a");
    ruling!(
        "Stone Haven Pilgrim",
        "Stone Haven Pilgrim gets +1/+1 just once, no matter how many artifacts and enchantments you control. It doesn’t get +2/+2 if you control an artifact and an enchantment."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Sigil of the Empty Throne");
    t.battlefield(P0, "Bow of Nylea");
    let pilgrim = pilgrim_attack(&mut t);
    t.resolve_all();
    assert_eq!(t.pt(pilgrim), (3, 3));
}

#[test]
fn moonlit_scavengers_returns_one_creature() {
    cr!("603.4", "115.1");
    ruling!(
        "Moonlit Scavengers",
        "Moonlit Scavengers's ability doesn't return multiple creatures if you control multiple artifacts and/or enchantments."
    );
    supported("Moonlit Scavengers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Sigil of the Empty Throne");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Moonlit Scavengers");
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant));
}

#[test]
fn moonlit_scavengers_checks_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Moonlit Scavengers",
        "Moonlit Scavengers's ability won't trigger at all if you don't control an artifact or enchantment immediately after it enters the battlefield. If you don't control an artifact or enchantment as the ability resolves, the ability has no effect. These don't have the be the same artifact or enchantment at both times, however."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Moonlit Scavengers");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Gone by resolution.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Moonlit Scavengers");
    t.settle();
    move_to(&mut t, thopter, Zone::Exile);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Replaced by a different enchantment.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Moonlit Scavengers");
    t.settle();
    move_to(&mut t, thopter, Zone::Exile);
    t.battlefield(P0, "Sigil of the Empty Throne");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Tethered Griffin (state trigger)
// ---------------------------------------------------------------------------------------

#[test]
fn tethered_griffin_doesnt_check_again_on_resolution() {
    cr!("603.8");
    ruling!(
        "Tethered Griffin",
        "Only checks if you control no enchantments at the time it triggers. It does not check again on resolution. Gaining control of an enchantment before then will not save the Griffin."
    );
    supported("Tethered Griffin");
    let mut t = TestGame::new(2);
    let griffin = t.battlefield(P0, "Tethered Griffin");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice"), 1);
    t.battlefield(P0, "Sigil of the Empty Throne");
    t.resolve_all();
    assert!(!t.on_battlefield(griffin));
}

#[test]
fn tethered_griffin_triggers_even_for_a_moment() {
    cr!("603.8", "608.2");
    ruling!(
        "Tethered Griffin",
        "The ability will trigger if you don’t control an enchantment, even for a brief moment during the resolution of another spell or ability."
    );
    supported("Scrollshift");
    // Scrollshift exiles P0's only enchantment and returns it.
    let mut t = TestGame::new(2);
    let sigil = t.battlefield(P0, "Sigil of the Empty Throne");
    let griffin = t.battlefield(P0, "Tethered Griffin");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    let card = in_hand_with_mana(&mut t, P0, "Scrollshift");
    t.cast(P0, card).target(Entity::Object(sigil)).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Sigil of the Empty Throne").len(), 1);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice"), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(griffin));
}

// ---------------------------------------------------------------------------------------
// Costs and choices
// ---------------------------------------------------------------------------------------

#[test]
fn thirst_for_meaning_discard_choices() {
    cr!("701.9a", "118.12");
    ruling!(
        "Thirst for Meaning",
        "You can discard either one enchantment card or two cards which may or may not be enchantments. If you really want to, you can discard two enchantment cards."
    );
    supported("Thirst for Meaning");
    let setup = || {
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &["Opt", "Opt", "Opt"]);
        let a = t.hand(P0, "Pacifism");
        let b = t.hand(P0, "Holy Strength");
        let card = in_hand_with_mana(&mut t, P0, "Thirst for Meaning");
        (t, card, a, b)
    };
    // One enchantment.
    let (mut t, card, a, _) = setup();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 2, "Pacifism and the spell");
    assert_eq!(t.hand_size(P0), 4);
    // Two cards (not enchantments).
    let (mut t, card, _, _) = setup();
    t.answer_yes(P0, false);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    // Two enchantment cards.
    let (mut t, card, a, b) = setup();
    t.answer_yes(P0, false);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Pacifism") && t.in_graveyard(P0, "Holy Strength"));
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn scrollshift_target_is_optional_but_an_illegal_one_stops_the_draw() {
    cr!("608.2b", "115.1");
    ruling!(
        "Scrollshift",
        "You don’t have to choose a target for Scrollshift. However, if you do, and that permanent is an illegal target at the time Scrollshift tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    // No target: draw.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    let card = in_hand_with_mana(&mut t, P0, "Scrollshift");
    t.answer_targets(P0, &[]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Target gone: no draw.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    let card = in_hand_with_mana(&mut t, P0, "Scrollshift");
    t.cast(P0, card).target(Entity::Object(bears)).go();
    move_to(&mut t, bears, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Scrollshift"));
}

#[test]
fn floodtide_serpent_returns_an_enchantment_each_attack() {
    cr!("508.1g", "508.1h", "508.1i");
    ruling!(
        "Floodtide Serpent",
        "You must return an enchantment you control to its owner’s hand during each combat in which you attack with Floodtide Serpent."
    );
    supported("Floodtide Serpent");
    // No enchantment: it can't attack.
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Floodtide Serpent");
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[serpent]));
    assert!(!t.g.is_attacking(t.g.current(serpent)));
    // With one: attacking returns it.
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Floodtide Serpent");
    let sigil = t.battlefield(P0, "Sigil of the Empty Throne");
    t.answer_choose(P0, &[Entity::Object(sigil)]);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[serpent]));
    assert!(t.g.is_attacking(t.g.current(serpent)));
    assert!(t.in_hand(P0, "Sigil of the Empty Throne"));
}
