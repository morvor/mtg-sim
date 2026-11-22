//! Rulings batch P190 — interactions of cards grouped by unique mana costs and evasion:
//! cast triggers that outlive their spell (CR 603.3), intervening "if" clauses and new
//! objects (CR 603.4, 400.7), triggered abilities that trigger an additional time
//! (Echoes of Eternity), Auras put onto the battlefield with other permanents (CR
//! 303.4f), characteristic-defining layers (CR 613), and the timing of gift triggers.

use crate::r_p076_common::{is_blocked, mana};
use crate::r_s01_common::{supported, tokens};
use crate::r_s03_common::to_blockers;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::{attach_new, damage};
use crate::r_s08_common::legal_cast_methods;
use crate::r_s11_common::{manifest_card, turn_face_up};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// From P1's end step to P0's upkeep, with P0's upkeep triggers on the stack.
fn to_upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

/// P1 casts Cancel targeting the spell `spell`, and it resolves (only Cancel).
fn p1_cancels(t: &mut TestGame, spell: ObjectId) {
    let cancel = t.hand(P1, "Cancel");
    mana(t, P1, ManaType::U, 2);
    mana(t, P1, ManaType::C, 1);
    t.cast(P1, cancel).target(spell).go();
    t.resolve();
}

fn asked_option(t: &TestGame, from: usize, prompt: &str) -> usize {
    t.asked()[from..]
        .iter()
        .filter(
            |(_, d)| matches!(d, Decision::ChooseOption { prompt: p, .. } if p.contains(prompt)),
        )
        .count()
}

#[test]
fn jolly_gerbils_triggers_when_the_gift_is_given() {
    cr!("702.174a", "702.174b", "603.2");
    ruling!(
        "Jolly Gerbils",
        "The ability of Jolly Gerbils triggers when the gift is actually given. For permanent spells, that happens when the gift triggered ability resolves. For instants and sorceries, that happens when the spell resolves."
    );
    supported("Jolly Gerbils");
    // An instant: nothing as it's cast; the trigger comes as it resolves.
    let mut t = TestGame::new(2);
    let gerbils = t.battlefield(P0, "Jolly Gerbils");
    let crumb = t.hand(P0, "Crumb and Get It");
    mana(&mut t, P0, ManaType::W, 1);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, crumb).target(gerbils).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.resolve();
    assert!(t.in_graveyard(P0, "Crumb and Get It"));
    assert_eq!(t.stack_len(), 1, "Jolly Gerbils triggered");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // A permanent spell: the gift is given as its gift trigger resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jolly Gerbils");
    let scrap = t.hand(P0, "Scrapshooter");
    mana(&mut t, P0, ManaType::G, 3);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, scrap).go();
    t.resolve();
    assert!(t.on_battlefield(scrap));
    // Only the gift trigger (Scrapshooter's other trigger has no target).
    assert_eq!(t.stack_len(), 1);
    let p1_hand = t.hand_size(P1);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P1), p1_hand + 1);
    assert_eq!(t.stack_len(), 1, "Jolly Gerbils triggered");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn departed_deckhands_ability_doesnt_unblock_a_blocked_creature() {
    cr!("509.1h", "506.4");
    ruling!(
        "Departed Deckhand",
        "Activating Departed Deckhand's last ability after a creature has become blocked by a non-Spirit creature won't cause that creature to become unblocked."
    );
    supported("Departed Deckhand");
    let mut t = TestGame::new(2);
    let deck = t.battlefield(P0, "Departed Deckhand");
    let attacker = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    to_blockers(
        &mut t,
        &[(attacker, Entity::Player(P1))],
        &[(blocker, attacker)],
    );
    assert!(is_blocked(&t, attacker));
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, deck, 0, &[obj(attacker)]).unwrap();
    t.resolve_all();
    assert!(is_blocked(&t, attacker));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn departed_deckhand_is_sacrificed_and_the_spell_does_nothing() {
    cr!("603.3", "608.2b");
    ruling!(
        "Departed Deckhand",
        "If the spell that targets Departed Deckhand has no other targets, it won't resolve (because it no longer has a legal target after Departed Deckhand is sacrificed)."
    );
    ruling!(
        "Departed Deckhand",
        "You'll sacrifice Departed Deckhand even if you counter the spell that targets it."
    );
    // The spell's only target is gone: it doesn't resolve.
    let mut t = TestGame::new(2);
    let deck = t.battlefield(P0, "Departed Deckhand");
    let growth = t.hand(P1, "Giant Growth");
    mana(&mut t, P1, ManaType::G, 1);
    let spell = t.cast(P1, growth).target(deck).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.zone(deck), Zone::Graveyard(P0));
    assert!(t.g.stack.contains(&spell));
    t.resolve();
    assert!(t.in_graveyard(P1, "Giant Growth"));
    assert!(t.g.stack.is_empty());
    // The spell is countered in response to the trigger: P0 still sacrifices it.
    let mut t = TestGame::new(2);
    let deck = t.battlefield(P0, "Departed Deckhand");
    let growth = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    let spell = t.cast(P0, growth).target(deck).go();
    t.settle();
    p1_cancels(&mut t, spell);
    assert!(t.in_graveyard(P0, "Giant Growth"));
    t.resolve_all();
    assert_eq!(t.zone(deck), Zone::Graveyard(P0));
}

#[test]
fn echoes_of_eternity_extra_trigger_makes_its_own_choices() {
    cr!("603.2d", "603.3d");
    ruling!(
        "Echoes of Eternity",
        "Echoes of Eternity's first ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability."
    );
    supported("Echoes of Eternity");
    supported("Pierce Strider");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Echoes of Eternity");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    enter(&mut t, P0, "Pierce Strider");
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 17);
}

#[test]
fn echoes_of_eternity_doesnt_affect_as_enters_abilities() {
    cr!("614.12", "603.2d");
    ruling!(
        "Echoes of Eternity",
        "Abilities that apply \"as [this permanent] enters the battlefield\" or \"as [this permanent] is turned face up\" are also unaffected."
    );
    supported("Adaptive Automaton");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoes of Eternity");
    let from = t.asked().len();
    enter(&mut t, P0, "Adaptive Automaton");
    assert_eq!(asked_option(&t, from, "creature type"), 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn echoes_of_eternity_turned_face_up_triggers_only_if_colorless_face_up() {
    cr!("603.2d", "708.8", "701.40b");
    ruling!(
        "Echoes of Eternity",
        "Abilities that apply \"when [this permanent] is turned face up\" will trigger an additional time only if that permanent is colorless once it has been turned face up."
    );
    supported("Proteus Machine");
    supported("Fathom Seer");
    // Proteus Machine is colorless face up: its ability triggers twice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoes of Eternity");
    let pm = manifest_card(&mut t, P0, "Proteus Machine");
    assert!(turn_face_up(&mut t, P0, pm));
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // Fathom Seer is blue face up (though colorless face down): once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoes of Eternity");
    t.lands(P0, "Island", 2);
    let fs = manifest_card(&mut t, P0, "Fathom Seer");
    assert!(turn_face_up(&mut t, P0, fs));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn act_of_aggression_can_target_an_untapped_creature() {
    cr!("115.1");
    ruling!(
        "Act of Aggression",
        "Act of Aggression can target any creature an opponent controls, even one that's untapped."
    );
    supported("Act of Aggression");
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P1, "Grizzly Bears");
    let c = t.hand(P0, "Act of Aggression");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, c).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj(bears).controller, P0);
    assert!(!t.obj(bears).tapped);
}

#[test]
fn eerie_ultimatum_aura_cant_enchant_a_permanent_entering_with_it() {
    cr!("303.4f", "303.4g");
    ruling!(
        "Eerie Ultimatum",
        "An Aura being put onto the battlefield this way can’t enchant anything else that is being put onto the battlefield at the same time."
    );
    supported("Eerie Ultimatum");
    supported("Rancor");
    let setup = |t: &mut TestGame| {
        let rancor = t.graveyard(P0, "Rancor");
        let bears = t.graveyard(P0, "Grizzly Bears");
        mana(t, P0, ManaType::W, 2);
        mana(t, P0, ManaType::B, 3);
        mana(t, P0, ManaType::G, 2);
        let c = t.hand(P0, "Eerie Ultimatum");
        t.answer_choose(P0, &[obj(rancor), obj(bears)]);
        t.cast(P0, c).go();
        t.resolve_all();
        (rancor, bears)
    };
    // A creature already on the battlefield: the Aura enchants it, not the Bears.
    let mut t = TestGame::new(2);
    let memnite = t.battlefield(P0, "Memnite");
    let (rancor, bears) = setup(&mut t);
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(rancor));
    assert_eq!(t.obj_now(rancor).attached_to, Some(obj(memnite)));
    // Nothing else to enchant: the Aura stays in the graveyard.
    let mut t = TestGame::new(2);
    let (rancor, bears) = setup(&mut t);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(rancor), Zone::Graveyard(P0));
}

#[test]
fn genesis_ultimatum_triggers_wait_until_it_is_exiled() {
    cr!("603.3", "608.2n");
    ruling!(
        "Genesis Ultimatum",
        "Any abilities that trigger as the permanents enter the battlefield this way won't be put onto the stack until after Genesis Ultimatum has finished resolving and is in exile."
    );
    ruling!(
        "Genesis Ultimatum",
        "Any abilities that trigger as the permanents enter the battlefield this way won’t be put onto the stack until after Genesis Ultimatum has finished resolving and is in exile."
    );
    supported("Genesis Ultimatum");
    let mut t = TestGame::new(2);
    let ev = t.library_top(P0, "Elvish Visionary");
    let c = t.hand(P0, "Genesis Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::U, 3);
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose(P0, &[obj(ev)]);
    let spell = t.cast(P0, c).go();
    t.g.resolve_top();
    // Resolved: the spell is in exile, the trigger is waiting to be put on the stack.
    assert_eq!(t.zone(spell), Zone::Exile);
    assert!(t.on_battlefield(ev));
    assert!(t.g.stack.is_empty());
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let src = match &t.obj(t.g.stack[0]).stack.as_ref().unwrap().kind {
        mtg_engine::object::StackKind::Triggered { source, .. } => Some(*source),
        _ => None,
    };
    assert_eq!(src, Some(t.g.current(ev)));
}

#[test]
fn harness_infinity_doesnt_discard() {
    cr!("701.9a", "701.12a");
    ruling!(
        "Harness Infinity",
        "Cards put into your graveyard this way are not discarded. Any ability that triggers when you discard a card will not trigger."
    );
    supported("Harness Infinity");
    supported("Bone Miser");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bone Miser");
    let bears = t.hand(P0, "Grizzly Bears");
    let gy = t.graveyard(P0, "Raging Goblin");
    let c = t.hand(P0, "Harness Infinity");
    mana(&mut t, P0, ManaType::B, 3);
    mana(&mut t, P0, ManaType::G, 3);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.zone(gy), Zone::Hand(P0));
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn daemogoth_woe_eater_sacrificed_to_its_own_ability_triggers() {
    cr!("603.6c", "603.10a", "701.21a");
    ruling!(
        "Daemogoth Woe-Eater",
        "Daemogoth Woe-Eater’s second ability will trigger if you sacrifice it as its first ability resolves."
    );
    supported("Daemogoth Woe-Eater");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Daemogoth Woe-Eater");
    t.hand(P1, "Grizzly Bears");
    to_upkeep(&mut t);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(d)]);
    t.resolve_all();
    assert_eq!(t.zone(d), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn giant_ambush_beetle_doesnt_have_to_attack() {
    cr!("509.1c");
    ruling!(
        "Giant Ambush Beetle",
        "Deciding to have the targeted creature block doesn't force you to attack with Giant Ambush Beetle that turn. If Giant Ambush Beetle doesn't attack, the targeted creature is free to block whichever creature its controller chooses, or block no creatures at all."
    );
    supported("Giant Ambush Beetle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    let beetle = enter(&mut t, P0, "Giant Ambush Beetle");
    t.resolve_all();
    // Only the Hill Giant attacks; the Bears block it.
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(bears, giant)]);
    assert!(is_blocked(&t, giant));
    assert!(t.on_battlefield(beetle));
    // Or block nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Giant Ambush Beetle");
    t.resolve_all();
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[]);
    assert!(!is_blocked(&t, giant));
}

#[test]
fn deity_of_scars_activated_in_advance() {
    cr!("602.2b", "701.19a", "122.3");
    ruling!(
        "Deity of Scars",
        "Deity of Scars's ability causes it to have more toughness when you activate the ability (as a result of losing a -1/-1 counter) and then puts a regeneration shield on it when the ability resolves."
    );
    supported("Deity of Scars");
    let mut t = TestGame::new(2);
    let deity = enter(&mut t, P0, "Deity of Scars");
    assert_eq!(t.pt(deity), (5, 5));
    mana(&mut t, P0, ManaType::B, 1);
    t.activate(P0, deity, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.pt(deity), (6, 6));
    t.resolve_all();
    // Dealt 5 damage, it survives without using the shield (it isn't tapped).
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 5, deity);
    assert!(t.on_battlefield(deity));
    assert!(!t.obj(deity).tapped);
    assert_eq!(t.obj(deity).damage, 5);
    // More damage: the shield is used.
    damage(&mut t, src, 1, deity);
    assert!(t.on_battlefield(deity));
    assert!(t.obj(deity).tapped);
    assert_eq!(t.obj(deity).damage, 0);
}

#[test]
fn devourer_of_destiny_cast_trigger_resolves_even_if_countered() {
    cr!("603.3");
    ruling!(
        "Devourer of Destiny",
        "Devourer of Destiny's last triggered ability will resolve before Devourer of Destiny does. If Devourer of Destiny is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    supported("Devourer of Destiny");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Devourer of Destiny");
    mana(&mut t, P0, ManaType::C, 7);
    let spell = t.cast(P0, c).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    p1_cancels(&mut t, spell);
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn demigod_of_revenge_countered_still_returns() {
    cr!("603.3", "608.2b");
    ruling!(
        "Demigod of Revenge",
        "If Demigod of Revenge is countered *before* its triggered ability resolves, the ability will still resolve. It will return that Demigod of Revenge from your graveyard to the battlefield, as well as any others."
    );
    supported("Demigod of Revenge");
    let mut t = TestGame::new(2);
    let other = t.graveyard(P0, "Demigod of Revenge");
    let c = t.hand(P0, "Demigod of Revenge");
    mana(&mut t, P0, ManaType::B, 5);
    let spell = t.cast(P0, c).go();
    t.settle();
    p1_cancels(&mut t, spell);
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(c));
    assert!(t.on_battlefield(other));
}

#[test]
fn leyline_of_the_guildpact_lands_get_every_basic_type() {
    cr!("305.7", "205.1b", "305.6");
    ruling!(
        "Leyline of the Guildpact",
        "Each land you control will have the land types Plains, Island, Swamp, Mountain, and Forest. They'll also have the mana ability of each basic land type"
    );
    ruling!(
        "Leyline of the Guildpact",
        "Giving a land additional basic land types doesn't change its name or whether it's legendary or basic."
    );
    // "Each nonland permanent you control is all colors." compiles too now.
    supported("Leyline of the Guildpact");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of the Guildpact");
    let mine = t.battlefield(P0, "Urza's Mine");
    let cradle = t.battlefield(P0, "Gaea's Cradle");
    let forest = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Urza's Mine");
    t.g.recompute();
    for l in [mine, cradle, forest] {
        for st in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            assert!(t.obj(l).chars.has_subtype(st), "{st}");
        }
    }
    assert!(t.obj(mine).chars.has_subtype("Urza's"));
    assert_eq!(t.obj(mine).chars.name, "Urza's Mine");
    assert!(!t.obj(mine).chars.supertypes.contains(Supertype::Basic));
    assert!(t
        .obj(cradle)
        .chars
        .supertypes
        .contains(Supertype::Legendary));
    assert!(t.obj(forest).chars.supertypes.contains(Supertype::Basic));
    assert_eq!(t.obj(forest).chars.name, "Forest");
    assert!(!t.obj(theirs).chars.has_subtype("Plains"));
    // Urza's Mine has its own mana ability and one of each basic land type's.
    let texts: Vec<String> = t
        .obj(mine)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .map(|a| a.text.to_string())
        .collect();
    assert_eq!(texts.len(), 6, "{texts:?}");
    for m in ["{W}", "{U}", "{B}", "{R}", "{G}"] {
        assert!(texts.iter().any(|x| x.contains(m)), "{m}: {texts:?}");
    }
}

#[test]
fn godhead_of_awe_timestamps() {
    cr!("613.4b", "613.4c", "613.4d", "613.7");
    ruling!(
        "Godhead of Awe",
        "Godhead of Awe's ability overwrites other effects that set a creature's power and toughness to specific values only if those effects existed before the Godhead entered."
    );
    supported("Godhead of Awe");
    supported("Sudden Spoiling");
    supported("Twisted Image");
    supported("Brute Strength");
    // An earlier setting effect is overwritten; the switch still applies last, after
    // the modifications; counters still apply.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spoil = t.hand(P0, "Sudden Spoiling");
    mana(&mut t, P0, ManaType::B, 3);
    t.cast(P0, spoil).target(Entity::Player(P1)).go();
    t.resolve_all();
    let brute = t.hand(P0, "Brute Strength");
    mana(&mut t, P0, ManaType::R, 2);
    t.cast(P0, brute).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    let twist = t.hand(P0, "Twisted Image");
    mana(&mut t, P0, ManaType::U, 1);
    t.cast(P0, twist).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    t.g.add_counters(obj(bears), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    // Godhead enters: base 1/1 (overwrites Spoiling's 0/2), +3/+1, +1/+1, switched.
    enter(&mut t, P0, "Godhead of Awe");
    assert_eq!(t.pt(bears), (3, 5));
    // A setting effect after the Godhead wins over it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    enter(&mut t, P0, "Godhead of Awe");
    assert_eq!(t.pt(bears), (1, 1));
    let spoil = t.hand(P0, "Sudden Spoiling");
    mana(&mut t, P0, ManaType::B, 3);
    t.cast(P0, spoil).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (0, 2));
}

#[test]
fn filigree_angel_counts_itself() {
    cr!("603.2", "608.2h");
    ruling!(
        "Filigree Angel",
        "Filigree Angel's \"enters\" ability counts Filigree Angel itself, assuming that it's still on the battlefield and still an artifact by the time the ability resolves."
    );
    supported("Filigree Angel");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    enter(&mut t, P0, "Filigree Angel");
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

#[test]
fn iname_as_one_put_onto_the_battlefield_doesnt_trigger() {
    cr!("603.4");
    ruling!(
        "Iname as One",
        "Effects that put Iname onto the battlefield don't cause its \"enters\" ability to trigger."
    );
    supported("Iname as One");
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Iname as One");
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn essenceknit_scholar_checks_as_the_end_step_starts() {
    cr!("603.4", "513.1");
    ruling!(
        "Essenceknit Scholar",
        "Essenceknit Scholar's last ability will check as your end step starts to see if a creature died under your control this turn. If none did, the ability won't trigger at all. It's not possible to cause a creature to die during your end step in time to have the ability trigger."
    );
    supported("Essenceknit Scholar");
    // No death before the end step: nothing, even if one dies during it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essenceknit Scholar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    assert_eq!(t.stack_len(), 0);
    let hand = t.hand_size(P0);
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), hand);
    // A creature died earlier in the turn: it draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essenceknit Scholar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.settle();
    t.set_step(P0, Step::PostcombatMain);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn firemane_angel_changing_zones_before_its_trigger_resolves() {
    cr!("603.4", "400.7");
    ruling!(
        "Firemane Angel",
        "If Firemane Angel is put into your graveyard from the battlefield or returned from your graveyard to the battlefield during your upkeep before its triggered ability resolves, you won't gain 1 life."
    );
    supported("Firemane Angel");
    // On the battlefield, then destroyed in response.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Firemane Angel");
    to_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(angel, None);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(angel), Zone::Graveyard(P0));
    assert_eq!(t.life(P0), 20);
    // In the graveyard, then returned in response.
    let mut t = TestGame::new(2);
    let angel = t.graveyard(P0, "Firemane Angel");
    to_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, angel, Zone::Battlefield);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(angel));
    assert_eq!(t.life(P0), 20);
    // Unmoved: P0 gains 1 life.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Firemane Angel");
    to_upkeep(&mut t);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn krond_must_be_enchanted_on_attack_and_resolution() {
    cr!("603.4");
    ruling!(
        "Krond the Dawn-Clad",
        "If Krond isn't enchanted when it's declared as an attacking creature, the ability won't trigger. If Krond isn't enchanted when the ability resolves, the ability won't do anything."
    );
    supported("Krond the Dawn-Clad");
    let mut t = TestGame::new(2);
    let krond = t.battlefield(P0, "Krond the Dawn-Clad");
    t.battlefield(P1, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(krond, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0);
    // Enchanted as it attacks, but not as the ability resolves.
    let mut t = TestGame::new(2);
    let krond = t.battlefield(P0, "Krond the Dawn-Clad");
    let rancor = attach_new(&mut t, P0, "Rancor", krond);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s01_common::attack_with(&mut t, &[(krond, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(rancor, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Still enchanted: the target is exiled.
    let mut t = TestGame::new(2);
    let krond = t.battlefield(P0, "Krond the Dawn-Clad");
    attach_new(&mut t, P0, "Rancor", krond);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s01_common::attack_with(&mut t, &[(krond, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn rona_countered_after_being_cast_from_the_graveyard_can_be_cast_again() {
    cr!("601.2a", "701.6a");
    ruling!(
        "Rona, Sheoldred's Faithful",
        "If Rona is countered or dies after being cast from your graveyard, it returns to the graveyard. It may be cast this way again later."
    );
    let mut t = TestGame::new(2);
    let rona = t.graveyard(P0, "Rona, Sheoldred's Faithful");
    for _ in 0..4 {
        t.hand(P0, "Shock");
    }
    let cast_it = |t: &mut TestGame| {
        mana(t, P0, ManaType::U, 1);
        mana(t, P0, ManaType::B, 2);
        mana(t, P0, ManaType::C, 1);
        let m = legal_cast_methods(t, P0, rona);
        let m = m
            .into_iter()
            .find(|m| matches!(m, CastMethod::Alternative(_)))
            .expect("castable from the graveyard");
        t.cast(P0, t.g.current(rona)).method(m).go()
    };
    let spell = cast_it(&mut t);
    p1_cancels(&mut t, spell);
    assert_eq!(t.zone(rona), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 2);
    cast_it(&mut t);
    t.resolve_all();
    assert!(t.on_battlefield(rona));
    assert_eq!(t.hand_size(P0), 0);
}
