//! CR 113: abilities — what abilities are, their categories, where they function, their
//! sources and controllers, and adding and removing abilities.

use super::r105_util::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn target_creature() -> TargetSpec {
    TargetSpec::object(Filter::creature(), "target creature")
}

/// A free instant: "Target creature [mods] until end of turn."
fn modify_instant(name: &str, mods: Vec<Modification>) -> CardDef {
    card_with(
        name,
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![target_creature()],
            Effect::Modify {
                what: Sel::Target(0),
                mods,
                duration: Duration::EndOfTurn,
            },
        )],
    )
}

/// Casts a free instant targeting `target` and resolves it.
fn cast_on(t: &mut TestGame, p: PlayerId, def: CardDef, target: ObjectId) {
    let c = put_in_hand(t, p, def);
    t.g.turn.priority = Some(p);
    t.cast(p, c).target(target).go();
    t.resolve();
    t.g.recompute();
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

fn gains(k: KeywordKind) -> Modification {
    Modification::AddKeyword(Keyword::new(k))
}

/// "An Aura with a static ability affecting the enchanted creature."
fn aura(name: &str, effect: StaticEffect) -> CardDef {
    let mut ab = vec![AbilityDef::new(
        AbilityKind::Keyword(Keyword::with_filter(
            KeywordKind::Enchant,
            Filter::creature(),
        )),
        "Enchant creature",
    )];
    ab.push(static_ab(effect));
    card_with(name, "{0}", "Enchantment — Aura", None, ab)
}

fn attach_aura(t: &mut TestGame, p: PlayerId, def: CardDef, to: ObjectId) -> ObjectId {
    let a = put(t, p, def);
    assert!(t.g.attach(a, Entity::Object(to)));
    t.g.recompute();
    a
}

#[test]
fn abilities_are_characteristics_defined_by_text_or_granted_by_effects() {
    cr!("113.1", "113.1a");
    let mut t = TestGame::new(2);
    // Defined by its rules text.
    let angel = t.battlefield(P0, "Serra Angel");
    assert!(has(&t, angel, KeywordKind::Flying) && has(&t, angel, KeywordKind::Vigilance));
    // Granted by an effect ("gains").
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.obj(bears).chars.abilities.is_empty());
    let leap = card_from_text(
        "Leap Test",
        "{0}",
        "Instant",
        None,
        "Target creature gains flying until end of turn.",
    );
    cast_on(&mut t, P0, leap, bears);
    assert!(has(&t, bears, KeywordKind::Flying));
    t.advance_to(P1, Step::Upkeep);
    assert!(!has(&t, bears, KeywordKind::Flying));
}

#[test]
fn a_player_can_have_an_ability() {
    cr!("113.1b");
    let mut t = TestGame::new(2);
    // Leyline of Sanctity: "You have hexproof."
    t.battlefield(P0, "Leyline of Sanctity");
    let theirs = t.hand(P1, "Lightning Bolt");
    let cands = spell_target_candidates(&t, P1, theirs, 0);
    assert!(!cands.contains(&Entity::Player(P0)));
    assert!(cands.contains(&Entity::Player(P1)));
    let mine = t.hand(P0, "Lightning Bolt");
    assert!(spell_target_candidates(&t, P0, mine, 0).contains(&Entity::Player(P0)));
}

#[test]
fn an_ability_on_the_stack_is_an_object() {
    cr!("113.1c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve();
    // Soul Warden's triggered ability is on the stack as an object.
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    assert_eq!(t.obj(trig).kind, ObjKind::StackAbility);
    assert_eq!(t.obj(trig).zone, Zone::Stack);
    let stifle = t.hand(P1, "Stifle");
    t.lands(P1, "Island", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, stifle).target(trig).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn abilities_affect_their_object_other_objects_and_players() {
    cr!("113.2");
    let mut t = TestGame::new(2);
    // Shivan Dragon's ability affects the Dragon itself.
    let dragon = t.battlefield(P0, "Shivan Dragon");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, dragon, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(dragon), (6, 5));
    // Glorious Anthem affects other objects.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Glorious Anthem");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    // Soul Warden affects a player.
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn an_ability_can_be_detrimental() {
    cr!("113.2a");
    let mut t = TestGame::new(2);
    // Hulking Goblin: "This creature can't block."
    let goblin = t.battlefield(P0, "Hulking Goblin");
    assert!(!t.g.can_block_at_all(goblin));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P0))], &[(goblin, bears)]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn an_additional_cost_is_an_ability_of_the_card() {
    cr!("113.2b");
    let mut t = TestGame::new(2);
    // Muraganda Petroglyphs: "Creatures with no abilities get +2/+2." Mardu Outrider's
    // only text is an additional cost to cast it — that's an ability.
    t.battlefield(P0, "Muraganda Petroglyphs");
    let outrider = t.battlefield(P0, "Mardu Outrider");
    let bear = t.battlefield(P0, "Runeclaw Bear");
    t.g.recompute();
    assert_eq!(t.pt(outrider), (5, 5));
    assert_eq!(t.pt(bear), (4, 4));
}

#[test]
fn each_instance_of_an_ability_functions_independently() {
    cr!("113.2c");
    let mut t = TestGame::new(2);
    // Each paragraph of a card's text is a separate ability.
    let cove = card("Tranquil Cove");
    assert_eq!(cove.front().chars.abilities.len(), 3);
    // A creature given the same triggered ability twice triggers twice.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let grant = || {
        modify_instant(
            "Grant",
            vec![Modification::AddAbility(triggered_ab(
                TriggerCond::Attacks(Filter::Source),
                gain_life(1),
            ))],
        )
    };
    cast_on(&mut t, P0, grant(), bears);
    cast_on(&mut t, P0, grant(), bears);
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn abilities_generate_one_shot_and_continuous_effects() {
    cr!("113.2d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Craw Wurm");
    // One-shot: Lightning Bolt deals damage once.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(bears).go();
    t.resolve();
    assert_eq!(t.obj(bears).damage, 3);
    // Continuous: Giant Growth lasts until end of turn.
    let growth = t.hand(P0, "Giant Growth");
    t.lands(P0, "Forest", 1);
    t.cast(P0, growth).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (9, 7));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bears), (6, 4));
    // A replacement effect: Tranquil Cove enters tapped.
    t.set_step(P1, Step::PrecombatMain);
    let cove = t.hand(P1, "Tranquil Cove");
    t.play_land(P1, cove).unwrap();
    assert!(t.obj(t.named_on_battlefield("Tranquil Cove")[0]).tapped);
}

#[test]
fn the_four_general_categories_of_abilities() {
    cr!("113.3");
    let kind = |name: &str| card(name).front().chars.abilities[0].kind.clone();
    assert!(matches!(kind("Lightning Bolt"), AbilityKind::Spell(_)));
    assert!(matches!(kind("Prodigal Pyromancer"), AbilityKind::Activated(_)));
    assert!(matches!(kind("Soul Warden"), AbilityKind::Triggered(_)));
    assert!(matches!(kind("Glorious Anthem"), AbilityKind::Static(_)));
}

#[test]
fn spell_abilities_are_followed_as_the_spell_resolves() {
    cr!("113.3a");
    let mut t = TestGame::new(2);
    // Abrupt Decay's text is a static ability ("This spell can't be countered") and a
    // spell ability.
    let abilities = card("Abrupt Decay").front().chars.abilities.clone();
    assert!(matches!(abilities[0].kind, AbilityKind::Static(_)));
    assert!(matches!(abilities[1].kind, AbilityKind::Spell(_)));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let decay = t.hand(P0, "Abrupt Decay");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.cast(P0, decay).target(bears).go();
    // Nothing happens until it resolves.
    assert!(t.on_battlefield(bears));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn activated_abilities_are_activated_with_priority_and_use_the_stack() {
    cr!("113.3b");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    // Without priority, it can't be activated.
    t.g.turn.priority = Some(P1);
    let uid = activated_uid(&t, pyro, 0);
    assert!(t.g.activate_ability(P0, pyro, uid).is_err());
    // With priority: the cost is paid and the ability goes on the stack.
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    assert!(t.obj(pyro).tapped);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P1), 20);
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn triggered_abilities_go_on_the_stack_the_next_time_a_player_would_receive_priority() {
    cr!("113.3c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve();
    // The Bears entered during resolution; the ability was put on the stack before
    // anyone got priority, and hasn't resolved yet.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P0), 20);
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn static_abilities_apply_while_the_permanent_is_on_the_battlefield() {
    cr!("113.3d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    // It doesn't use the stack.
    assert_eq!(t.stack_len(), 0);
    let disenchant = t.hand(P1, "Disenchant");
    t.lands(P1, "Plains", 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, disenchant).target(anthem).go();
    t.resolve();
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn mana_abilities_dont_use_the_stack() {
    cr!("113.4");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elves, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool_count(&t, P0, ManaType::G), 1);
    // They can be activated while paying a cost: the Elves help cast Grizzly Bears.
    t.g.players[0].mana_pool.mana.clear();
    t.g.objects[elves.0 as usize].tapped = false;
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert!(t.obj(elves).tapped);
}

#[test]
fn loyalty_abilities_follow_special_timing_rules() {
    cr!("113.5");
    let mut t = TestGame::new(2);
    let elspeth = t.battlefield(P0, "Elspeth, Knight-Errant");
    // Not while the stack isn't empty.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(P1).go();
    assert!(t.activate(P0, elspeth, 0, &[]).is_err());
    t.resolve();
    // Main phase, empty stack, your turn: once.
    t.activate(P0, elspeth, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert!(t.activate(P0, elspeth, 0, &[]).is_err());
    // Not during another player's turn.
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.activate(P0, elspeth, 0, &[]).is_err());
}

#[test]
fn abilities_of_permanents_function_only_on_the_battlefield() {
    cr!("113.6");
    let mut t = TestGame::new(2);
    // Soul Warden in a graveyard and Glorious Anthem in a hand do nothing.
    t.graveyard(P0, "Soul Warden");
    t.hand(P0, "Glorious Anthem");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.pt(t.named_on_battlefield("Grizzly Bears")[0]), (2, 2));
    // An instant's spell ability does something only while it's resolving on the stack:
    // Lightning Bolt in a graveyard does nothing.
    t.graveyard(P0, "Lightning Bolt");
    t.settle();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn an_ability_that_says_where_it_doesnt_function_functions_everywhere_else() {
    cr!("113.6c");
    let mut t = TestGame::new(2);
    // A card whose static ability "doesn't function in graveyards".
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::creature().you_control(),
        mods: vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
    });
    s.zone = FunctionZone::AnywhereExcept(ZoneKind::Graveyard);
    let def = card_with(
        "Everywhere Banner",
        "{2}",
        "Artifact",
        None,
        vec![AbilityDef::new(AbilityKind::Static(s), "static")],
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    let banner = put_in_hand(&mut t, P0, def);
    t.g.recompute();
    // It works from the hand...
    assert_eq!(t.pt(bears), (3, 3));
    // ... and from exile ...
    let exiled = t
        .g
        .move_object(banner, Zone::Exile, mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    // ... but not from a graveyard.
    t.g.move_object(
        exiled,
        Zone::Graveyard(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn abilities_that_modify_what_an_object_costs_function_as_it_is_cast() {
    cr!("113.6d");
    let mut t = TestGame::new(2);
    // Affinity for artifacts reduces Frogmite's cost while it's being cast.
    for _ in 0..4 {
        t.battlefield(P0, "Ornithopter");
    }
    let frog = t.hand(P0, "Frogmite");
    t.cast(P0, frog).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Frogmite").len(), 1);
    // Mardu Outrider's additional cost must be paid as it's cast.
    let outrider = t.hand(P0, "Mardu Outrider");
    let fodder = t.hand(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.cast(P0, outrider).go();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn a_casting_restriction_functions_where_the_object_could_be_cast() {
    cr!("113.6e");
    let mut t = TestGame::new(2);
    let def = || {
        card_from_text(
            "Combat Trick",
            "{0}",
            "Instant",
            None,
            "Cast this spell only during combat.\nTarget creature gets +2/+2 until end of turn.",
        )
    };
    let bears = t.battlefield(P0, "Grizzly Bears");
    let trick = put_in_hand(&mut t, P0, def());
    assert!(t.cast(P0, trick).target(bears).try_go().is_err());
    t.set_step(P0, Step::BeginningOfCombat);
    t.cast(P0, trick).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn an_ability_changing_where_an_object_can_be_cast_from_functions_there() {
    cr!("113.6f");
    let mut t = TestGame::new(2);
    // Gravecrawler: "You may cast this card from your graveyard as long as you control a
    // Zombie."
    let crawler = t.graveyard(P0, "Gravecrawler");
    t.lands(P0, "Swamp", 2);
    assert!(t.cast(P0, crawler).try_go().is_err());
    t.battlefield(P0, "Walking Corpse");
    t.cast(P0, crawler).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Gravecrawler").len(), 1);
}

#[test]
fn cant_be_countered_functions_on_the_stack() {
    cr!("113.6g");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let decay = t.hand(P0, "Abrupt Decay");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    let spell = t.cast(P0, decay).target(bears).go();
    let cs = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    // Abrupt Decay is still on the stack and resolves.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

/// "Return target card from your graveyard to the battlefield [with N +1/+1 counters]."
fn reanimate(counters: u32) -> CardDef {
    let mut to = Destination::battlefield();
    if counters > 0 {
        to.with_counters = vec![(counters::PLUS1.into(), Value::c(counters as i32))];
    }
    card_with(
        "Return Test",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(
                Filter::and(vec![
                    Filter::Card,
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::OwnedBy(PlayerRel::You),
                ]),
                "target card from your graveyard",
            )],
            Effect::Move {
                what: Sel::Target(0),
                to,
            },
        )],
    )
}

#[test]
fn an_ability_modifying_how_its_object_enters_functions_as_it_enters() {
    cr!("113.6h");
    let mut t = TestGame::new(2);
    let cove = t.graveyard(P0, "Tranquil Cove");
    let r = put_in_hand(&mut t, P0, reanimate(0));
    t.cast(P0, r).target(cove).go();
    t.resolve();
    assert!(t.on_battlefield(cove));
    assert!(t.obj_now(cove).tapped);
}

#[test]
fn counters_cant_be_put_on_it_functions_as_it_enters() {
    cr!("113.6i");
    let mut t = TestGame::new(2);
    // Tatterkite: "This creature can't have counters put on it."
    let kite = t.graveyard(P0, "Tatterkite");
    let r = put_in_hand(&mut t, P0, reanimate(2));
    t.cast(P0, r).target(kite).go();
    t.resolve();
    assert!(t.on_battlefield(kite));
    assert_eq!(t.counters(kite, counters::PLUS1), 0);
    // A creature without the ability gets them.
    let bears = t.graveyard(P0, "Grizzly Bears");
    let r = put_in_hand(&mut t, P0, reanimate(2));
    t.cast(P0, r).target(bears).go();
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn an_ability_with_a_cost_payable_only_from_hand_functions_from_hand() {
    cr!("113.6j");
    let mut t = TestGame::new(2);
    // Simian Spirit Guide: "Exile this card from your hand: Add {R}."
    let guide = t.hand(P0, "Simian Spirit Guide");
    t.activate(P0, guide, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, ManaType::R), 1);
    assert!(t.in_exile("Simian Spirit Guide"));
    // On the battlefield the cost can't be paid, and the ability doesn't function.
    let on_bf = t.battlefield(P0, "Simian Spirit Guide");
    assert!(t.activate(P0, on_bf, 0, &[]).is_err());
}

#[test]
fn an_ability_moving_its_object_out_of_a_zone_functions_only_there() {
    cr!("113.6m");
    let mut t = TestGame::new(2);
    // Reassembling Skeleton: "{1}{B}: Return this card from your graveyard to the
    // battlefield tapped."
    let on_bf = t.battlefield(P0, "Reassembling Skeleton");
    t.lands(P0, "Swamp", 4);
    assert!(t.activate(P0, on_bf, 0, &[]).is_err());
    let skel = t.graveyard(P0, "Reassembling Skeleton");
    t.activate(P0, skel, 0, &[]).unwrap();
    t.resolve();
    assert!(t.on_battlefield(skel));
    assert!(t.obj_now(skel).tapped);
}

#[test]
fn deck_construction_abilities_function_before_the_game_begins() {
    cr!("113.6n");
    use mtg_engine::deck::{check_constructed, DeckProblem};
    let deck = |cards: &[(&str, usize)]| {
        let mut d = Vec::new();
        for (n, k) in cards {
            d.extend(std::iter::repeat_n(card(n), *k));
        }
        d
    };
    // Relentless Rats: "A deck can have any number of cards named Relentless Rats."
    assert!(check_constructed(&deck(&[("Relentless Rats", 30), ("Swamp", 30)])).is_empty());
    // Other cards are limited to four.
    let bad = check_constructed(&deck(&[("Grizzly Bears", 5), ("Forest", 55)]));
    assert_eq!(
        bad,
        vec![DeckProblem::TooManyCopies {
            name: "Grizzly Bears".into(),
            have: 5,
            max: 4
        }]
    );
    // Seven Dwarves: "A deck can have up to seven cards named Seven Dwarves."
    assert!(check_constructed(&deck(&[("Seven Dwarves", 7), ("Mountain", 53)])).is_empty());
    assert_eq!(
        check_constructed(&deck(&[("Seven Dwarves", 8), ("Mountain", 52)])),
        vec![DeckProblem::TooManyCopies {
            name: "Seven Dwarves".into(),
            have: 8,
            max: 7
        }]
    );
}

#[test]
fn abilities_of_command_zone_objects_function_there() {
    cr!("113.6p");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // An emblem.
    let maker = card_with(
        "Emblem Maker",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![],
            Effect::CreateEmblem {
                abilities: vec![static_ab(StaticEffect::Continuous {
                    affected: Filter::creature().you_control(),
                    mods: vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
                })],
            },
        )],
    );
    let m = put_in_hand(&mut t, P0, maker);
    t.cast(P0, m).go();
    t.resolve();
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    // A vanguard: Titania lets its owner play an additional land.
    t.command(P0, "Titania");
    let f1 = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Forest");
    t.play_land(P0, f1).unwrap();
    t.play_land(P0, f2).unwrap();
    // A face-up conspiracy.
    let consp = card_from_text(
        "Test Conspiracy",
        "",
        "Conspiracy",
        None,
        "Creatures you control get +1/+1.",
    );
    t.custom(P0, consp, Zone::Command);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    // A face-down scheme (still in the scheme deck) has no functioning abilities.
    let scheme = card_from_text(
        "Test Scheme",
        "",
        "Ongoing Scheme",
        None,
        "Creatures you control get +1/+1.",
    );
    let s = t.custom(P0, scheme, Zone::Command);
    t.g.objects[s.0 as usize].face_down = true;
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    t.g.objects[s.0 as usize].face_down = false;
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn the_controller_of_an_ability_on_the_stack() {
    cr!("113.8");
    let mut t = TestGame::new(2);
    // An activated ability is controlled by the player who activated it: P0 activates
    // P1's (stolen) Prodigal Pyromancer.
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    let treason = t.hand(P0, "Act of Treason");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, treason).target(pyro).go();
    t.resolve();
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    let ab = t.g.stack[0];
    assert_eq!(t.obj(ab).controller, P0);
    assert_eq!(t.obj(t.g.current(pyro)).owner, P1);
    t.resolve();
    // A triggered ability of a card that has no controller (in a graveyard) is controlled
    // by the card's owner.
    t.graveyard(P1, "Bloodghast");
    t.set_step(P1, Step::PrecombatMain);
    let swamp = t.hand(P1, "Swamp");
    t.play_land(P1, swamp).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.obj(t.g.stack[0]).controller, P1);
}

#[test]
fn abilities_on_the_stack_arent_spells() {
    cr!("113.9");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    let ab = t.g.stack[0];
    // Counterspell ("counter target spell") can't target it; Stifle can.
    let cs = t.hand(P1, "Counterspell");
    assert!(!spell_target_candidates(&t, P1, cs, 0).contains(&Entity::Object(ab)));
    let stifle = t.hand(P1, "Stifle");
    assert!(spell_target_candidates(&t, P1, stifle, 0).contains(&Entity::Object(ab)));
    // A static ability doesn't use the stack: Glorious Anthem's effect applies with
    // nothing to counter.
    t.resolve();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Glorious Anthem");
    assert_eq!(t.stack_len(), 0);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn effects_add_and_remove_abilities() {
    cr!("113.10");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_on(&mut t, P0, modify_instant("Gain", vec![gains(KeywordKind::Flying)]), bears);
    assert!(has(&t, bears, KeywordKind::Flying));
    let angel = t.battlefield(P1, "Serra Angel");
    cast_on(
        &mut t,
        P0,
        modify_instant(
            "Lose",
            vec![Modification::RemoveKeyword(KeywordKind::Flying)],
        ),
        angel,
    );
    assert!(!has(&t, angel, KeywordKind::Flying));
    assert!(has(&t, angel, KeywordKind::Vigilance));
}

#[test]
fn an_added_activated_ability_keeps_its_activation_instructions() {
    cr!("113.10a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Target creature gains '{T}: You gain 1 life. Activate only as a sorcery.'"
    let mut act = ActivatedAbility::new(Cost::tap(), Body::effect(gain_life(1)));
    act.timing = ActivationTiming::Sorcery;
    let granted = AbilityDef::new(
        AbilityKind::Activated(act),
        "{T}: You gain 1 life. Activate only as a sorcery.",
    );
    cast_on(
        &mut t,
        P0,
        modify_instant("Grant", vec![Modification::AddAbility(granted)]),
        bears,
    );
    t.g.turn.step = Step::BeginningOfCombat;
    assert!(t.activate(P0, bears, 0, &[]).is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, bears, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn removing_an_ability_removes_all_instances_of_it() {
    cr!("113.10b");
    let mut t = TestGame::new(2);
    // Serra Angel has flying, and an Aura gives it flying again.
    let angel = t.battlefield(P0, "Serra Angel");
    let flight = t.battlefield(P0, "Flight");
    assert!(t.g.attach(flight, Entity::Object(angel)));
    t.g.recompute();
    let n = t
        .obj(angel)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Keyword(k) if k.kind == KeywordKind::Flying))
        .count();
    assert_eq!(n, 2);
    cast_on(
        &mut t,
        P0,
        modify_instant(
            "Lose",
            vec![Modification::RemoveKeyword(KeywordKind::Flying)],
        ),
        angel,
    );
    assert!(!has(&t, angel, KeywordKind::Flying));
}

#[test]
fn the_most_recent_of_adding_and_removing_an_ability_prevails() {
    cr!("113.10c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let gain = || modify_instant("Gain", vec![gains(KeywordKind::Flying)]);
    let lose = || {
        modify_instant(
            "Lose",
            vec![Modification::RemoveKeyword(KeywordKind::Flying)],
        )
    };
    // Gain, then lose: no flying.
    cast_on(&mut t, P0, gain(), a);
    cast_on(&mut t, P0, lose(), a);
    assert!(!has(&t, a, KeywordKind::Flying));
    // Lose, then gain: flying.
    cast_on(&mut t, P0, lose(), b);
    cast_on(&mut t, P0, gain(), b);
    assert!(has(&t, b, KeywordKind::Flying));
}

#[test]
fn an_object_that_cant_have_an_ability_doesnt_get_it() {
    cr!("113.11");
    let mut t = TestGame::new(2);
    // Archetype of Courage: "Creatures your opponents control lose first strike and can't
    // have or gain first strike."
    let knight = t.battlefield(P1, "Youthful Knight");
    assert!(has(&t, knight, KeywordKind::FirstStrike));
    t.battlefield(P0, "Archetype of Courage");
    t.g.recompute();
    assert!(!has(&t, knight, KeywordKind::FirstStrike));
    // A later effect can't give it first strike, but the rest of that effect applies.
    let trick = card_from_text(
        "Trick",
        "{0}",
        "Instant",
        None,
        "Target creature gets +2/+2 and gains first strike until end of turn.",
    );
    cast_on(&mut t, P1, trick, knight);
    assert!(!has(&t, knight, KeywordKind::FirstStrike));
    assert_eq!(t.pt(knight), (4, 3));
    // Neither can a first strike counter.
    t.g.objects[knight.0 as usize]
        .counters
        .insert("first strike".into(), 1);
    t.g.recompute();
    assert!(!has(&t, knight, KeywordKind::FirstStrike));
    // Creatures of the Archetype's controller aren't affected.
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(has(&t, mine, KeywordKind::FirstStrike));
}

#[test]
fn setting_a_characteristic_or_a_quality_isnt_granting_an_ability() {
    cr!("113.12");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muraganda Petroglyphs");
    // Enchanted creature has flying: an ability, so no bonus.
    let a = t.battlefield(P0, "Runeclaw Bear");
    let flight = t.battlefield(P0, "Flight");
    assert!(t.g.attach(flight, Entity::Object(a)));
    // Enchanted creature is red: a characteristic.
    let b = t.battlefield(P0, "Runeclaw Bear");
    attach_aura(
        &mut t,
        P0,
        aura(
            "Red Coat",
            StaticEffect::Continuous {
                affected: Filter::AttachedToSource,
                mods: vec![Modification::SetColors(cs("R"))],
            },
        ),
        b,
    );
    // Enchanted creature can't be blocked: a quality.
    let c = t.battlefield(P0, "Runeclaw Bear");
    attach_aura(
        &mut t,
        P0,
        aura(
            "Unseen Path",
            StaticEffect::Restriction(Restriction::CantBeBlocked(Filter::AttachedToSource)),
        ),
        c,
    );
    t.g.recompute();
    assert_eq!(t.pt(a), (2, 2));
    assert_eq!(t.pt(b), (4, 4));
    assert_eq!(t.obj(b).chars.colors, cs("R"));
    assert_eq!(t.pt(c), (4, 4));
}
