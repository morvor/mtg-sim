//! Rulings batch P044 — old legendary creatures and other one-offs grouped with them:
//! evasion exceptions (CR 702.14), activation zones (CR 113.6, 602.2), type-changing
//! effects and characteristic-defining abilities (CR 205.1, 604.3, 613.1d), the legend rule
//! (CR 704.5j), detain (CR 701.35), attack restrictions (CR 508.1c), and abilities that
//! stop activated abilities (CR 602.5).

use crate::r_p044_common::*;
use crate::r_s01_common::*;
use mtg_engine::ability::Filter;
use mtg_engine::card::card;
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn attacking(t: &TestGame, id: ObjectId) -> bool {
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    let id = t.g.current(id);
    t.g.matches(id, &Filter::Attacking, &ctx)
}

fn blocked_by(t: &TestGame, attacker: ObjectId) -> bool {
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    let id = t.g.current(attacker);
    t.g.matches(id, &Filter::Blocked, &ctx)
}

/// Whether `p` could activate any ability of `source` now.
fn can_activate(t: &mut TestGame, p: PlayerId, source: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let source = t.g.current(source);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source: s, .. } if *s == source))
}

#[test]
fn walk_exceptions_apply_to_every_blocker() {
    cr!("702.14c", "509.1b");
    ruling!(
        "Ur-Drago",
        "Allows any creature controlled by any player to block Swampwalking creatures as if they did not have this ability."
    );
    ruling!(
        "Lord Magnus",
        "The statement that creatures with Plainswalk or Forestwalk can be blocked applies to all creatures on the battlefield blocking them"
    );
    for (lord, walker, land) in [
        ("Ur-Drago", "Bog Wraith", "Swamp"),
        ("Lord Magnus", "Shanodin Dryads", "Forest"),
        ("Lord Magnus", "Zodiac Rooster", "Plains"),
    ] {
        supported(lord);
        supported(walker);
        // The attacker's controller controls the lord; the defender's Bears block.
        let mut t = TestGame::new(2);
        t.battlefield(P0, lord);
        let w = t.battlefield(P0, walker);
        t.lands(P1, land, 1);
        let bears = t.battlefield(P1, "Grizzly Bears");
        attack_with(&mut t, &[(w, Entity::Player(P1))]);
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, w)]));
        t.advance_to(P0, Step::DeclareBlockers);
        assert!(blocked_by(&t, w), "{lord} / {walker}");
        // Without the lord, the walker can't be blocked.
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, walker);
        t.lands(P1, land, 1);
        let bears = t.battlefield(P1, "Grizzly Bears");
        attack_with(&mut t, &[(w, Entity::Player(P1))]);
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, w)]));
        t.advance_to(P0, Step::DeclareBlockers);
        assert!(!blocked_by(&t, w), "{walker} alone");
    }
}

#[test]
fn abilities_that_work_only_on_the_battlefield() {
    cr!("113.6", "602.2");
    ruling!(
        "Arcanis the Omnipotent",
        "Arcanis's last ability can be activated only while it's on the battlefield."
    );
    ruling!(
        "Balthor the Defiled",
        "You can only activate this ability if Balthor the Defiled is on the battlefield."
    );
    supported("Arcanis the Omnipotent");
    supported("Balthor the Defiled");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    t.lands(P0, "Swamp", 3);
    for (name, zone) in [
        ("Arcanis the Omnipotent", Zone::Hand(P0)),
        ("Arcanis the Omnipotent", Zone::Graveyard(P0)),
        ("Balthor the Defiled", Zone::Graveyard(P0)),
        ("Balthor the Defiled", Zone::Hand(P0)),
    ] {
        let c = place(&mut t, P0, name, zone);
        assert!(!can_activate(&mut t, P0, c), "{name} in {zone:?}");
    }
    let a = t.battlefield(P0, "Arcanis the Omnipotent");
    assert!(can_activate(&mut t, P0, a));
    t.activate(P0, a, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Arcanis the Omnipotent"));
}

#[test]
fn rashka_gets_its_bonus_once_for_several_black_creatures() {
    cr!("603.2c", "509.1a");
    ruling!(
        "Rashka the Slayer",
        "Gets the bonus only once even if it blocks more than one black creature."
    );
    supported("Rashka the Slayer");
    supported("Brave the Sands");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Vampire Interloper");
    let b = t.battlefield(P0, "Vampire Interloper");
    t.battlefield(P1, "Brave the Sands");
    let rashka = t.battlefield(P1, "Rashka the Slayer");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(rashka, a), (rashka, b)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(blocked_by(&t, a) && blocked_by(&t, b));
    t.resolve_all();
    assert_eq!(t.pt(rashka), (4, 5));
}

#[test]
fn squee_must_be_in_the_graveyard_as_the_upkeep_begins() {
    cr!("603.2", "502.4", "503.1a");
    ruling!(
        "Squee, Goblin Nabob",
        "If Squee isn't in your graveyard as your upkeep begins, its ability won't trigger."
    );
    supported("Squee, Goblin Nabob");
    // In the graveyard: it triggers and returns.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Squee, Goblin Nabob");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Squee, Goblin Nabob"));
    // In the hand as the upkeep begins: nothing (no priority before the upkeep).
    let mut t = TestGame::new(2);
    t.hand(P0, "Squee, Goblin Nabob");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn rayne_triggers_per_target_and_checks_enchanted_on_resolution() {
    cr!("603.2c", "603.4", "608.2", "303.4");
    ruling!(
        "Rayne, Academy Chancellor",
        "If a spell or ability targets multiple permanents you control or targets you and one or more of your permanents, this triggers once for each such target."
    );
    ruling!(
        "Rayne, Academy Chancellor",
        "The \"if\" in the card text does not have a special meaning. You check if this card is enchanted when the ability resolves and draw 1 or 2 cards."
    );
    ruling!(
        "Rayne, Academy Chancellor",
        "When casting an Aura on this card, the triggered ability will resolve before the Aura enters."
    );
    supported("Rayne, Academy Chancellor");
    supported("Electrolyze");
    // Electrolyze targets P0 and P0's Giant: two triggers; P0 draws two (plus Electrolyze
    // draws for P1).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rayne, Academy Chancellor");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Electrolyze");
    let e = t.hand(P1, "Electrolyze");
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.cast(P1, e)
        .targets(&[Entity::Player(P0), Entity::Object(giant)])
        .go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw"), 2);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // P1 casts Pacifism on Rayne: the trigger resolves first, while Rayne isn't enchanted.
    let mut t = TestGame::new(2);
    let rayne = t.battlefield(P0, "Rayne, Academy Chancellor");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    cast_spell(&mut t, P1, "Pacifism", &[Entity::Object(rayne)]);
    assert_eq!(t.g.attachments_of(Entity::Object(rayne)).len(), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Enchanted by the time it resolves: two cards.
    let mut t = TestGame::new(2);
    let rayne = t.battlefield(P0, "Rayne, Academy Chancellor");
    t.set_step(P1, Step::PrecombatMain);
    cast_only(&mut t, P1, "Shock", &[Entity::Object(rayne)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw"), 1);
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(rayne));
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn blind_seer_color_change_carries_over_to_the_permanent() {
    cr!("400.7", "611.2c");
    ruling!(
        "Blind Seer",
        "If you change the color of a spell which is to become a permanent, the permanent will retain the color change until the end of the turn."
    );
    supported("Blind Seer");
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Blind Seer");
    t.lands(P0, "Island", 2);
    let bears = cast_only(&mut t, P0, "Grizzly Bears", &[]);
    let spell = t.g.current(bears);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, seer, 0, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(spell).chars.colors, ColorSet::single(Color::Blue));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).chars.colors, ColorSet::single(Color::Blue));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bears).chars.colors, ColorSet::single(Color::Green));
}

/// P0's upkeep with Hakim on the battlefield and Islands for its ability.
fn hakim_upkeep() -> (TestGame, ObjectId) {
    supported("Hakim, Loreweaver");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hakim, Loreweaver");
    t.lands(P0, "Island", 4);
    t.set_step(P0, Step::Upkeep);
    (t, h)
}

#[test]
fn hakim_returns_only_auras_that_can_enchant_him() {
    cr!("303.4a", "303.4g", "608.2b");
    ruling!(
        "Hakim, Loreweaver",
        "If you pick an Aura that can't legally enchant this card, then the Aura stays in the graveyard."
    );
    ruling!(
        "Hakim, Loreweaver",
        "You only check if he has no Auras when activating the ability. You can use the ability multiple times in response to each other to get multiple Auras on him this way."
    );
    // Wild Growth (enchant land) stays in the graveyard.
    let (mut t, h) = hakim_upkeep();
    let wg = t.graveyard(P0, "Wild Growth");
    t.activate(P0, h, 0, &[Entity::Object(wg)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Wild Growth"));
    // Two activations in response to each other: both Auras end up on Hakim.
    let (mut t, h) = hakim_upkeep();
    let a = t.graveyard(P0, "Holy Strength");
    let b = t.graveyard(P0, "Unholy Strength");
    t.activate(P0, h, 0, &[Entity::Object(a)]).unwrap();
    t.activate(P0, h, 0, &[Entity::Object(b)]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.attachments_of(Entity::Object(h)).len(), 2);
    assert_eq!(t.pt(h), (5, 7));
    // Now he's enchanted: no more activations.
    t.graveyard(P0, "Holy Strength");
    t.lands(P0, "Island", 2);
    assert!(!t.g.legal_actions(P0).iter().any(|x| matches!(x,
        Action::Activate { source, ability } if *source == h
            && *ability == t.obj_now(h).chars.abilities.iter().find(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_))).unwrap().uid)));
}

#[test]
fn phage_must_be_cast_from_your_hand() {
    cr!("603.4", "601.2");
    ruling!(
        "Phage the Untouchable",
        "It is only considered \"cast from your hand\" if you cast it as a spell from your hand. Putting it onto the battlefield from your hand using a spell or ability will cause you to lose the game."
    );
    supported("Phage the Untouchable");
    // Put onto the battlefield from the hand: P0 loses.
    let mut t = TestGame::new(3);
    let p = t.hand(P0, "Phage the Untouchable");
    t.g.move_object(p, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.resolve_all();
    assert!(t.has_lost(P0));
    // Cast from the hand: fine.
    let mut t = TestGame::new(3);
    cast_spell(&mut t, P0, "Phage the Untouchable", &[]);
    assert!(!t.has_lost(P0));
    assert_eq!(t.named_on_battlefield("Phage the Untouchable").len(), 1);
}

#[test]
fn mistform_ultimus_is_every_creature_type() {
    cr!("205.3m", "604.3", "613.1d", "613.1f", "613.3");
    ruling!(
        "Mistform Ultimus",
        "The type “Illusion” is printed on the card’s type line purely for flavor. The Ultimus has every other creature type as well."
    );
    ruling!(
        "Mistform Ultimus",
        "Mistform Ultimus has all creature types even if an effect removes its ability."
    );
    ruling!(
        "Mistform Ultimus",
        "Like any creature, if Mistform Ultimus becomes another type as the result of a spell or ability, it will lose all its other creature types."
    );
    supported("Mistform Ultimus");
    supported("Sudden Spoiling");
    supported("Imagecrafter");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mistform Ultimus");
    for ty in ["Illusion", "Elf", "Goblin", "Sliver"] {
        assert!(t.obj_now(m).chars.has_subtype(ty), "{ty}");
    }
    // Every creature type in the graveyard too.
    let g = t.graveyard(P0, "Mistform Ultimus");
    t.g.recompute();
    assert!(t.obj_now(g).chars.has_subtype("Wizard"));
    // Sudden Spoiling: it loses its abilities, but keeps all creature types.
    t.set_step(P1, Step::PrecombatMain);
    cast_spell(&mut t, P1, "Sudden Spoiling", &[Entity::Player(P0)]);
    assert_eq!(t.pt(m), (0, 2));
    assert!(t.obj_now(m).chars.has_subtype("Elf"));
    // Imagecrafter: it becomes the chosen type only.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mistform Ultimus");
    let ic = t.battlefield(P0, "Imagecrafter");
    t.activate(P0, ic, 0, &[Entity::Object(m)]).unwrap();
    t.resolve_all();
    let o = t.obj_now(m);
    assert!(!o.chars.all_creature_types);
    assert_eq!(o.chars.subtypes.len(), 1, "{:?}", o.chars.subtypes);
}

#[test]
fn jolrael_animates_the_lands_the_player_controls_on_resolution() {
    cr!("611.2c", "205.1b", "613.1d");
    ruling!(
        "Jolrael, Empress of Beasts",
        "Only affects lands the target player controls when the ability resolves."
    );
    ruling!("Jolrael, Empress of Beasts", "The lands do not get a creature type.");
    ruling!("Jolrael, Empress of Beasts", "The lands retain their abilities.");
    supported("Jolrael, Empress of Beasts");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jolrael, Empress of Beasts");
    t.lands(P0, "Forest", 3);
    crate::r_p045_common::bears_in_hand(&mut t, P0, 2);
    let forest = t.lands(P1, "Forest", 1)[0];
    t.activate(P0, j, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let o = t.obj_now(forest);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert_eq!(t.pt(forest), (3, 3));
    assert_eq!(
        o.chars.subtypes.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        vec!["Forest"]
    );
    assert!(!o.chars.abilities.is_empty(), "it can still tap for {{G}}");
    // A land P1 gets afterwards isn't a creature.
    let late = t.lands(P1, "Plains", 1)[0];
    t.g.recompute();
    assert!(!t.obj_now(late).is(CardType::Creature));
}

#[test]
fn raksha_and_sun_quan_affect_themselves() {
    cr!("109.2", "613.1f", "702.31b");
    ruling!(
        "Raksha Golden Cub",
        "Raksha Golden Cub is a Cat, so it also gets +2/+2 and double strike if it’s equipped."
    );
    ruling!("Sun Quan, Lord of Wu", "Sun Quan’s ability affects Sun Quan itself.");
    supported("Raksha Golden Cub");
    supported("Sun Quan, Lord of Wu");
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Raksha Golden Cub");
    let eq = t.battlefield(P0, "Bonesplitter");
    t.g.attach(eq, Entity::Object(r));
    t.g.recompute();
    assert_eq!(t.pt(r), (7, 6));
    assert!(t.obj_now(r).has_keyword(KeywordKind::DoubleStrike));
    let s = t.battlefield(P0, "Sun Quan, Lord of Wu");
    t.g.recompute();
    assert!(t.obj_now(s).has_keyword(KeywordKind::Horsemanship));
}

#[test]
fn the_legend_rule_needs_identical_names() {
    cr!("704.5j", "201.2");
    ruling!(
        "Ertai, Wizard Adept",
        "The Legend rule only applies to two legendary permanents with identical names."
    );
    ruling!(
        "Ertai, the Corrupted",
        "The Legend rule only applies to two legendary permanents with identical names."
    );
    ruling!(
        "Balthor the Stout",
        "The Legend rule only applies to two legendary permanents with identical names."
    );
    ruling!(
        "Kamahl, Pit Fighter",
        "The Legend rule only applies to two legendary permanents with identical names."
    );
    for (a, b, both) in [
        ("Ertai, Wizard Adept", "Ertai, the Corrupted", true),
        ("Balthor the Stout", "Balthor the Defiled", true),
        ("Kamahl, Pit Fighter", "Kamahl, Fist of Krosa", true),
        ("Kamahl, Pit Fighter", "Kamahl, Pit Fighter", false),
    ] {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P0, a);
        let y = t.battlefield(P0, b);
        t.settle();
        assert_eq!(t.on_battlefield(x) && t.on_battlefield(y), both, "{a} / {b}");
    }
}

#[test]
fn ixidor_turns_face_up_without_the_morph_cost() {
    cr!("702.37e", "708.8");
    ruling!(
        "Ixidor, Reality Sculptor",
        "When you turn a card face up, its controller does not have to pay its Morph cost."
    );
    supported("Ixidor, Reality Sculptor");
    let mut t = TestGame::new(2);
    let ix = t.battlefield(P0, "Ixidor, Reality Sculptor");
    let ego = t.hand(P1, "Scornful Egotist");
    t.lands(P1, "Wastes", 3);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, ego)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let fd = t.g.current(ego);
    // Face-down creatures get +1/+1.
    assert_eq!(t.pt(fd), (3, 3));
    // P1 has no mana left; Ixidor turns it face up anyway.
    t.lands(P0, "Island", 3);
    t.activate(P0, ix, 0, &[Entity::Object(fd)]).unwrap();
    t.resolve_all();
    let o = t.obj_now(fd);
    assert!(!o.face_down);
    assert_eq!(o.chars.name.as_str(), "Scornful Egotist");
}

#[test]
fn solkanar_doesnt_trigger_on_its_own_casting() {
    cr!("603.2", "603.6a");
    ruling!("Sol'kanar the Swamp King", "Will not trigger off itself being cast.");
    supported("Sol'kanar the Swamp King");
    let mut t = TestGame::new(2);
    cast_spell(&mut t, P0, "Sol'kanar the Swamp King", &[]);
    assert_eq!(t.life(P0), 20);
    cast_spell(&mut t, P1, "Dark Ritual", &[]);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn rith_payment_is_chosen_as_the_ability_resolves() {
    cr!("608.2c", "608.2d", "117.1");
    ruling!(
        "Rith, the Awakener",
        "You choose whether to pay {2}{G} as Rith's ability resolves."
    );
    supported("Rith, the Awakener");
    let mut t = TestGame::new(2);
    let rith = t.battlefield(P0, "Rith, the Awakener");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Forest", 3);
    // The payment is asked while the ability is resolving: nothing else happens in
    // between (no priority).
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. }),
        |g| g.stack.len(),
    );
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    attack_with(&mut t, &[(rith, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(!seen.lock().unwrap().is_empty());
    // Three green permanents (Rith, the Bears and P1's Elves): three Saprolings.
    assert_eq!(with_subtype(&t, P0, "Saproling").len(), 3);
}

#[test]
fn sightless_brawler_cant_attack_alone() {
    cr!("508.1c", "702.103b");
    ruling!(
        "Sightless Brawler",
        "Sightless Brawler or the creature it enchants can be declared as an attacker only if another creature is declared as an attacker at the same time."
    );
    supported("Sightless Brawler");
    // Alone: not declared.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Sightless Brawler");
    attack_with(&mut t, &[(b, Entity::Player(P1))]);
    assert!(!attacking(&t, b));
    // With another creature: both attack.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Sightless Brawler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(b, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    assert!(attacking(&t, b) && attacking(&t, bears));
    // Bestowed on the Bears: the Bears can't attack alone.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    let sb = t.hand(P0, "Sightless Brawler");
    t.cast(P0, sb)
        .target(bears)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 4));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!attacking(&t, bears));
}

#[test]
fn reality_chip_top_card_stays_in_the_library() {
    cr!("401.5", "305.1", "305.2", "702.34a");
    ruling!(
        "The Reality Chip",
        "The top card of your library is still in your library and not in your hand."
    );
    ruling!(
        "The Reality Chip",
        "You must pay all costs and follow all timing rules for cards played from the top of your library this way."
    );
    supported("The Reality Chip");
    supported("Boseiju, Who Endures");
    let mut t = TestGame::new(2);
    let chip = t.battlefield(P0, "The Reality Chip");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach_as_creature(chip, Entity::Object(bears));
    t.g.recompute();
    let top = t.library_top(P0, "Boseiju, Who Endures");
    t.lands(P0, "Forest", 2);
    t.g.turn.priority = Some(P0);
    // Its channel ability (discard it from your hand) can't be activated from the library.
    assert!(!can_activate(&mut t, P0, top));
    // Not on the opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!t
        .g
        .legal_actions(P0)
        .contains(&Action::PlayLand { card: top }));
    // During P0's main phase: yes; then no second land.
    t.set_step(P0, Step::PrecombatMain);
    t.play_land(P0, top).unwrap();
    let next = t.library_top(P0, "Forest");
    assert!(t.play_land(P0, next).is_err());
}

#[test]
fn activated_ability_stoppers_leave_triggered_abilities_alone() {
    cr!("602.5", "113.3c", "113.3d");
    ruling!(
        "Collector Ouphe",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities (such as equip) are activated abilities and will have a colon in their reminder text. Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected."
    );
    ruling!(
        "Edifice of Authority",
        "Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected by the last ability of Edifice of Authority. Notably, exerting a creature isn’t an activated ability."
    );
    supported("Collector Ouphe");
    supported("Edifice of Authority");
    // Collector Ouphe: no equip; Ichor Wellspring still triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Collector Ouphe");
    let eq = t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    assert!(!can_activate(&mut t, P0, eq));
    let hand = t.hand_size(P0);
    t.enter(P0, "Ichor Wellspring");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Edifice of Authority's last ability on Prodigal Sorcerer and Pitchburn Devils: the
    // Sorcerer's ability can't be activated; the Devils' dies trigger still works.
    let mut t = TestGame::new(2);
    let ed = t.battlefield(P1, "Edifice of Authority");
    t.g.add_counters(Entity::Object(ed), "brick", 3, None);
    let devils = t.battlefield(P0, "Pitchburn Devils");
    let sorc = t.battlefield(P0, "Prodigal Sorcerer");
    t.lands(P1, "Wastes", 2);
    t.set_step(P1, Step::End);
    t.activate(P1, ed, 1, &[Entity::Object(sorc)]).unwrap();
    t.resolve_all();
    t.g.untap(ed);
    t.activate(P1, ed, 1, &[Entity::Object(devils)]).unwrap();
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P0, sorc));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    kill(&mut t, devils);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn edifice_cant_undo_an_attack_or_an_activation() {
    cr!("508.1", "602.2", "117.3c");
    ruling!(
        "Edifice of Authority",
        "Edifice of Authority can’t be used to undo an attack or block once it has been declared."
    );
    ruling!(
        "Edifice of Authority",
        "Once a player has announced that they are activating the ability of a creature, Edifice of Authority can’t be used to undo it."
    );
    // Already attacking: "can't attack this turn" doesn't remove it from combat.
    let mut t = TestGame::new(2);
    let ed = t.battlefield(P1, "Edifice of Authority");
    t.lands(P1, "Wastes", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.activate(P1, ed, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(attacking(&t, bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    // Prodigal Sorcerer's ability already on the stack resolves; P0 can also respond to
    // the Edifice ability by activating it.
    let mut t = TestGame::new(2);
    let ed = t.battlefield(P1, "Edifice of Authority");
    t.g.add_counters(Entity::Object(ed), "brick", 3, None);
    t.lands(P1, "Wastes", 1);
    let sorc = t.battlefield(P0, "Prodigal Sorcerer");
    t.set_step(P1, Step::End);
    t.activate(P1, ed, 1, &[Entity::Object(sorc)]).unwrap();
    t.activate(P0, sorc, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    t.g.untap(sorc);
    assert!(!can_activate(&mut t, P0, sorc));
}

#[test]
fn detain_targets_can_have_different_controllers() {
    cr!("701.35a", "115.3");
    ruling!(
        "Azorius Justiciar",
        "The two creatures can be controlled by the same opponent or by different opponents."
    );
    ruling!(
        "Archon of the Triumvirate",
        "The two nonland permanents can be controlled by the same opponent or by different opponents."
    );
    supported("Azorius Justiciar");
    supported("Archon of the Triumvirate");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Prodigal Sorcerer");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.enter(P0, "Azorius Justiciar");
    t.resolve_all();
    t.set_step(P2, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P2, b));
    // Archon: an artifact of P1 and a creature of P2.
    let mut t = TestGame::new(3);
    let archon = t.battlefield(P0, "Archon of the Triumvirate");
    let ring = t.battlefield(P1, "Mind Stone");
    let sorc = t.battlefield(P2, "Prodigal Sorcerer");
    t.answer_targets(P0, &[Entity::Object(ring), Entity::Object(sorc)]);
    attack_with(&mut t, &[(archon, Entity::Player(P1))]);
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P1, ring));
    t.set_step(P2, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P2, sorc));
}

#[test]
fn lessons_can_be_in_the_main_deck() {
    cr!("100.2a", "205.3k");
    ruling!(
        "Containment Breach",
        "Lesson cards can be included in your main deck like other instant or sorcery cards."
    );
    ruling!(
        "Pest Summoning",
        "Lesson cards can be included in your main deck like other instant or sorcery cards."
    );
    supported("Containment Breach");
    supported("Pest Summoning");
    let mut deck = vec![card("Containment Breach"); 4];
    deck.extend(vec![card("Pest Summoning"); 4]);
    deck.extend(vec![card("Forest"); 52]);
    assert!(mtg_engine::deck::check_constructed(&deck).is_empty());
    // And they're cast from the main deck like any sorcery.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 1);
    let ps = t.hand(P0, "Pest Summoning");
    t.cast(P0, ps).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Pest").len(), 2);
}
