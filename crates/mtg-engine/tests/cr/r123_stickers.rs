//! CR 123: stickers.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::*;
use mtg_engine::stickers::{self, SheetFormat, StickerDef, StickerKind, StickerSheet};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn sheet(name: &str, stickers: Vec<StickerDef>) -> StickerSheet {
    StickerSheet {
        name: name.into(),
        stickers,
    }
}

fn name_st(word: &str, ticket_cost: u32) -> StickerDef {
    StickerDef {
        kind: StickerKind::Name(word.into()),
        ticket_cost,
    }
}

fn art() -> StickerDef {
    StickerDef {
        kind: StickerKind::Art,
        ticket_cost: 0,
    }
}

fn pt_st(p: i32, t: i32, ticket_cost: u32) -> StickerDef {
    StickerDef {
        kind: StickerKind::PowerToughness(p, t),
        ticket_cost,
    }
}

fn flying_st() -> StickerDef {
    StickerDef {
        kind: StickerKind::Ability(vec![AbilityDef::new(
            AbilityKind::Keyword(Keyword::new(KeywordKind::Flying)),
            "Flying",
        )]),
        ticket_cost: 0,
    }
}

/// Gives `p` these sticker sheets as in limited play.
fn limited(t: &mut TestGame, p: PlayerId, sheets: Vec<StickerSheet>) {
    stickers::choose_sheets(&mut t.g, p, sheets, SheetFormat::Limited).unwrap();
}

/// `p` puts the `choice`th available sticker (of `kind`) on `obj`.
fn put(
    t: &mut TestGame,
    p: PlayerId,
    obj: ObjectId,
    kind: Option<StickerType>,
    choice: usize,
) -> bool {
    t.answer(p, DecisionKind::Option, Answer::Index(choice));
    let r = stickers::put_from_sheets(&mut t.g, p, obj, kind, None, false);
    // The answer isn't used if there was only one sticker to choose.
    t.clear_answers();
    t.g.recompute();
    r
}

fn tickets(t: &TestGame, p: PlayerId) -> u32 {
    t.player(p).counter(counters::TICKET)
}

#[test]
fn a_sticker_is_a_marker_not_an_object_a_counter_or_a_token() {
    cr!("123.1");
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Big", vec![pt_st(5, 5, 0)])]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let objects = t.g.objects.len();
    assert!(put(&mut t, P0, bears, None, 0));
    assert_eq!(t.pt(bears), (5, 5));
    // No object was created, and it isn't a counter.
    assert_eq!(t.g.objects.len(), objects);
    assert!(t.obj(bears).counters.is_empty());
    // Changes from stickers aren't part of the copiable values: a Clone copying the
    // Bears is a 2/2.
    assert_eq!(t.obj(bears).copiable.power, Some(2));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let clone = t.enter(P0, "Clone");
    t.g.recompute();
    assert_eq!(t.obj(clone).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(clone), (2, 2));
}

#[test]
fn sticker_sheets_are_chosen_before_play() {
    cr!("123.2", "123.2a", "123.2b");
    let sheets = |n: usize| -> Vec<StickerSheet> {
        (0..n)
            .map(|i| sheet(&format!("Sheet {i}"), vec![name_st("Space", 0)]))
            .collect()
    };
    let mut t = TestGame::new(2);
    // Constructed: at least ten sheets, each unique; three are chosen at random.
    assert!(stickers::choose_sheets(&mut t.g, P0, sheets(9), SheetFormat::Constructed).is_err());
    let mut dup = sheets(10);
    dup[9].name = "Sheet 0".into();
    assert!(stickers::choose_sheets(&mut t.g, P0, dup, SheetFormat::Constructed).is_err());
    let chosen =
        stickers::choose_sheets(&mut t.g, P0, sheets(12), SheetFormat::Constructed).unwrap();
    assert_eq!(chosen.len(), 3);
    assert_eq!(stickers::sheets_of(&t.g, P0).len(), 3);
    // Limited: up to three sheets from the player's pool.
    assert!(stickers::choose_sheets(&mut t.g, P1, sheets(4), SheetFormat::Limited).is_err());
    assert!(stickers::choose_sheets(&mut t.g, P1, sheets(2), SheetFormat::Limited).is_ok());
    assert_eq!(stickers::sheets_of(&t.g, P1).len(), 2);
}

#[test]
fn players_have_access_only_to_the_stickers_on_their_chosen_sheets() {
    cr!("123.2c");
    let mut t = TestGame::new(2);
    limited(
        &mut t,
        P0,
        vec![sheet("Mine", vec![name_st("Space", 0), art()])],
    );
    limited(&mut t, P1, vec![sheet("Theirs", vec![pt_st(9, 9, 0)])]);
    // P0 has access to exactly the two stickers on their sheet; the sheets stay revealed
    // (anyone can see them).
    let options = stickers::available(&t.g, P0, P0, None, None, false);
    assert_eq!(options.len(), 2);
    assert!(options.iter().all(|s| s.player == P0));
    assert_eq!(stickers::sheets_of(&t.g, P1)[0].name, "Theirs");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!put(
        &mut t,
        P0,
        bears,
        Some(StickerType::PowerToughness),
        0
    ));
}

#[test]
fn a_player_chooses_a_sticker_not_already_on_an_object_they_own() {
    cr!("123.3", "123.3a");
    let mut t = TestGame::new(2);
    // Two stickers with the same word are two different stickers.
    limited(
        &mut t,
        P0,
        vec![sheet(
            "Twins",
            vec![name_st("Space", 0), name_st("Space", 0)],
        )],
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(put(&mut t, P0, bears, None, 0));
    assert_eq!(
        stickers::available(&t.g, P0, P0, None, None, false).len(),
        1
    );
    assert!(put(&mut t, P0, elves, None, 0));
    // Both are on objects P0 owns: none is left.
    let ogre = t.battlefield(P0, "Gray Ogre");
    assert!(!put(&mut t, P0, ogre, None, 0));
    // The Bears go to their owner's hand; the sticker isn't on anything anymore.
    t.g.move_object(bears, Zone::Hand(P0), MoveCause::Effect, Some(P0));
    assert!(put(&mut t, P0, ogre, None, 0));
    // An effect: Minotaur de Force — "you get {TK}, then you may put a sticker on a
    // nonland permanent you own".
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Pets", vec![art()])]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Minotaur de Force");
    t.settle();
    t.resolve_all();
    assert_eq!(tickets(&t, P0), 1);
    assert!(stickers::has_sticker(&t.g, bears, Some(StickerType::Art)));
}

#[test]
fn a_player_cant_put_a_sticker_on_an_object_they_dont_own() {
    cr!("123.3b");
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Mine", vec![art()])]);
    // P0 controls this creature but P1 owns it.
    let borrowed = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[borrowed.0 as usize].controller = P0;
    t.g.recompute();
    assert!(!put(&mut t, P0, borrowed, None, 0));
    assert!(!stickers::is_stickered(&t.g, borrowed));
}

#[test]
fn a_stickers_ticket_cost_is_paid_by_the_owner_and_not_again_when_moved() {
    cr!("123.3c", "123.3d");
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Pricey", vec![pt_st(6, 6, 2)])]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Player(P0), counters::TICKET, 1, None);
    // One ticket isn't enough for a ticket cost of 2.
    assert!(stickers::available(&t.g, P0, P0, None, None, false).is_empty());
    assert!(!put(&mut t, P0, bears, None, 0));
    t.g.add_counters(Entity::Player(P0), counters::TICKET, 1, None);
    assert!(put(&mut t, P0, bears, None, 0));
    assert_eq!(tickets(&t, P0), 0);
    assert_eq!(t.pt(bears), (6, 6));
    // Moving it to another object doesn't cost tickets again.
    let effect = *t.g.stickers.last().unwrap();
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(stickers::move_sticker(&mut t.g, effect, elves));
    t.g.recompute();
    assert_eq!(t.pt(elves), (6, 6));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(tickets(&t, P0), 0);
}

#[test]
fn a_stickered_object_has_a_sticker_on_it_now() {
    cr!("123.4");
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Mine", vec![art()])]);
    // Big Winner: "This creature has trample as long as you control a stickered
    // permanent."
    let winner = t.battlefield(P0, "Big Winner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!t.obj(winner).has_keyword(KeywordKind::Trample));
    assert!(put(&mut t, P0, bears, None, 0));
    assert!(t.obj(winner).has_keyword(KeywordKind::Trample));
    // Once it has no sticker on it, it isn't stickered, though it had one before.
    let in_hand =
        t.g.move_object(bears, Zone::Hand(P0), MoveCause::Effect, Some(P0))
            .unwrap();
    let back =
        t.g.move_object(in_hand, Zone::Battlefield, MoveCause::Effect, Some(P0))
            .unwrap();
    t.g.recompute();
    assert!(!stickers::is_stickered(&t.g, back));
    assert!(!t.obj(winner).has_keyword(KeywordKind::Trample));
}

#[test]
fn stickers_stay_on_objects_moving_to_public_zones_only() {
    cr!("123.5");
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Big", vec![pt_st(4, 4, 0)])]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(put(&mut t, P0, bears, None, 0));
    // To the graveyard (public): the sticker applies to the new object.
    let dead =
        t.g.move_object(bears, Zone::Graveyard(P0), MoveCause::Effect, Some(P0))
            .unwrap();
    t.g.recompute();
    assert!(stickers::is_stickered(&t.g, dead));
    assert_eq!(t.obj(dead).chars.power, Some(4));
    // Into the library (hidden): it doesn't.
    let lib =
        t.g.move_object(dead, Zone::Library(P0), MoveCause::Effect, Some(P0))
            .unwrap();
    t.g.recompute();
    assert!(!stickers::is_stickered(&t.g, lib));
    assert_eq!(t.obj(lib).chars.power, Some(2));
}

#[test]
fn name_stickers_add_a_word_where_the_controller_chooses() {
    cr!("123.6", "123.6a", "123.6b", "123.6c");
    // "Wolf in _____ Clothing" has three words: the blank isn't one.
    assert_eq!(stickers::word_count("Wolf in _____ Clothing"), 3);
    assert_eq!(stickers::word_count("Grizzly Bears-Cubs"), 2);
    let mut t = TestGame::new(2);
    limited(
        &mut t,
        P0,
        vec![sheet(
            "Words",
            vec![name_st("Sheep's", 0), name_st("Big", 0)],
        )],
    );
    let wolf = t.battlefield(P0, "Wolf in _____ Clothing");
    // The controller chooses: at the beginning or after any number of words.
    let log = spy(&mut t, P0, |_g, _p, d| match d {
        mtg_engine::decision::Decision::ChooseOption {
            prompt, options, ..
        } if prompt.contains("name sticker's word") => Some(options.join(" | ")),
        _ => None,
    });
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // the "Sheep's" sticker
    t.answer(P0, DecisionKind::Option, Answer::Index(2)); // after two words
    assert!(stickers::put_from_sheets(
        &mut t.g, P0, wolf, None, None, false
    ));
    t.g.recompute();
    assert_eq!(t.obj(wolf).chars.name, "Wolf in Sheep's _____ Clothing");
    // The choices: the beginning, or after one, two or three of its words.
    assert_eq!(
        probe_lines(&log),
        vec![[
            "Sheep's Wolf in _____ Clothing",
            "Wolf Sheep's in _____ Clothing",
            "Wolf in Sheep's _____ Clothing",
            "Wolf in _____ Clothing Sheep's",
        ]
        .join(" | ")]
    );
    // It's a text-changing effect: with a later effect that sets the name, the sticker
    // still applies in timestamp order... and a nameless object's name becomes the word.
    let fd = t.battlefield(P0, "Grizzly Bears");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, fd));
    t.g.recompute();
    assert!(put(&mut t, P0, fd, None, 0));
    assert_eq!(t.obj(fd).chars.name, "Big");
}

#[test]
fn letters_and_unique_vowels_on_a_name_sticker() {
    cr!("123.6d", "123.6e");
    assert_eq!(stickers::letter_count("Oddball Orb", 'o'), 2);
    assert_eq!(stickers::unique_vowels("Yeehaw"), 3);
    assert_eq!(stickers::unique_vowels("AAAaaa"), 1);
    // _____-o-saurus: "When this creature enters, you may put a name sticker on it. Put a
    // +1/+1 counter on it for each unique vowel on that sticker."
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Words", vec![name_st("Yeehaw", 0)])]);
    t.answer_yes(P0, true);
    let saurus = t.enter(P0, "_____-o-saurus");
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(saurus, counters::PLUS1), 3);
    // The number of o's in name stickers on a permanent (uppercase or lowercase).
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Words", vec![name_st("Oboe", 0)])]);
    let pinger = CB::new("Letter Counter")
        .enchantment()
        .ability(trig(
            TriggerCond::Custom("sticker placed:self".into()),
            Body::effect(Effect::DealDamage {
                source: Sel::This,
                amount: Value::Custom("sticker letter:o".into()),
                to: Sel::Players(PlayerRef::EachOpponent),
            }),
        ))
        .build();
    let e = t.custom(P0, pinger, Zone::Battlefield);
    assert!(put(&mut t, P0, e, None, 0));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn ability_stickers_grant_their_abilities() {
    cr!("123.7", "123.7a");
    let mut t = TestGame::new(2);
    limited(
        &mut t,
        P0,
        vec![sheet("Wings", vec![flying_st(), flying_st()])],
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(put(&mut t, P0, bears, None, 0));
    assert!(t.obj(bears).has_keyword(KeywordKind::Flying));
    // On a card in a zone other than the battlefield, too.
    let dead = t.graveyard(P0, "Llanowar Elves");
    assert!(put(&mut t, P0, dead, None, 0));
    assert!(t.obj(dead).has_keyword(KeywordKind::Flying));
    // The ability of an ability sticker is the one it grants, even when the object
    // doesn't have it because of another effect.
    t.g.effects.push(ContinuousEffect {
        id: 999,
        source: None,
        controller: P1,
        timestamp: 10_000,
        duration: Duration::Permanent,
        affected: Affected::Objects(vec![bears]),
        mods: vec![Modification::RemoveAllAbilities],
        layer1: None,
        created_turn: 1,
    });
    t.g.dirty = true;
    t.g.recompute();
    assert!(!t.obj(bears).has_keyword(KeywordKind::Flying));
    let abilities = stickers::ability_sticker_abilities(&t.g, bears);
    assert!(abilities
        .iter()
        .any(|a| a.keyword().is_some_and(|k| k.kind == KeywordKind::Flying)));
}

use mtg_engine::game::{Affected, ContinuousEffect};

#[test]
fn power_and_toughness_stickers_set_power_and_toughness() {
    cr!("123.8", "123.8a");
    let mut t = TestGame::new(2);
    limited(
        &mut t,
        P0,
        vec![sheet(
            "Stats",
            vec![
                pt_st(5, 1, 0),
                pt_st(1, 5, 0),
                pt_st(7, 7, 0),
                name_st("Big", 0),
            ],
        )],
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(put(&mut t, P0, bears, None, 0));
    assert_eq!(t.pt(bears), (5, 1));
    // The later sticker takes precedence (timestamp order).
    assert!(put(&mut t, P0, bears, None, 0));
    assert_eq!(t.pt(bears), (1, 5));
    // On a Vehicle card in a zone other than the battlefield.
    let copter = t.graveyard(P0, "Smuggler's Copter");
    assert!(put(
        &mut t,
        P0,
        copter,
        Some(StickerType::PowerToughness),
        0
    ));
    assert_eq!(t.obj(copter).chars.power, Some(7));
    // The power of a sticker is the value printed on a power and toughness sticker;
    // other stickers (the name sticker) have none. Effects on the creature don't change
    // it.
    assert!(put(&mut t, P0, bears, Some(StickerType::Name), 0));
    let ctx = mtg_engine::eval::Ctx::new(Some(bears), P0);
    assert_eq!(
        t.g.eval_value(&Value::Custom("sticker power".into()), &ctx),
        1 + 5
    );
    assert_eq!(
        stickers::sticker_power_toughness(&t.g, bears),
        vec![(5, 1), (1, 5)]
    );
}

#[test]
fn an_art_sticker_is_only_a_marker() {
    cr!("123.9");
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Art", vec![art()])]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let before = format!("{:?}", t.obj(bears).chars);
    assert!(put(&mut t, P0, bears, None, 0));
    assert_eq!(format!("{:?}", t.obj(bears).chars), before);
    // Spells and abilities can identify it.
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t
        .g
        .matches(bears, &Filter::HasSticker(Some(StickerType::Art)), &ctx));
    assert!(stickers::is_stickered(&t.g, bears));
}

/// Graf Rats and Midnight Scavengers meld into Chittering Host (5/6) at the beginning of
/// P0's combat.
fn meld_host(t: &mut TestGame) -> ObjectId {
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.advance_to_step(mtg_engine::turn::Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    t.named_on_battlefield("Chittering Host")[0]
}

#[test]
fn stickers_on_melded_cards_are_on_the_melded_permanent_in_their_order() {
    cr!("123.5a");
    let mut t = TestGame::new(2);
    limited(
        &mut t,
        P0,
        vec![sheet("Stats", vec![pt_st(1, 9, 0), pt_st(8, 2, 0)])],
    );
    let rats = t.battlefield(P0, "Graf Rats");
    let scavengers = t.battlefield(P0, "Midnight Scavengers");
    // The 1/9 sticker goes on Graf Rats first, then the 8/2 on Midnight Scavengers.
    assert!(put(&mut t, P0, rats, None, 0));
    assert!(put(&mut t, P0, scavengers, None, 0));
    let host = meld_host(&mut t);
    // Both stickers are on Chittering Host; the later one takes precedence.
    assert_eq!(stickers::stickers_on(&t.g, host).len(), 2);
    assert_eq!(t.pt(host), (8, 2));
    // In the other order, the other one does.
    let mut t = TestGame::new(2);
    limited(
        &mut t,
        P0,
        vec![sheet("Stats", vec![pt_st(1, 9, 0), pt_st(8, 2, 0)])],
    );
    let rats = t.battlefield(P0, "Graf Rats");
    let scavengers = t.battlefield(P0, "Midnight Scavengers");
    assert!(put(&mut t, P0, scavengers, None, 1));
    assert!(put(&mut t, P0, rats, None, 0));
    let host = meld_host(&mut t);
    assert_eq!(t.pt(host), (1, 9));
}

#[test]
fn a_sticker_on_a_mutating_spell_is_on_the_merged_permanent() {
    cr!("123.5b");
    let mut t = TestGame::new(2);
    limited(&mut t, P0, vec![sheet("Big", vec![pt_st(9, 9, 0)])]);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    // Gemrazer gets a sticker, then is exiled (a public zone: the sticker stays), and P0
    // may cast it from exile.
    let gem = t.battlefield(P0, "Gemrazer");
    assert!(put(&mut t, P0, gem, None, 0));
    let gem =
        t.g.move_object(gem, Zone::Exile, MoveCause::Effect, Some(P0))
            .unwrap();
    t.g.recompute();
    assert!(stickers::is_stickered(&t.g, gem));
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![gem],
        Duration::EndOfTurn,
        false,
        None,
    );
    // It mutates under Grizzly Bears: the merged permanent is the Bears, with the sticker.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.players[P0.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::G, 3);
    t.cast(P0, gem)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve();
    t.g.recompute();
    assert_eq!(t.obj(bears).chars.name, "Grizzly Bears");
    assert!(stickers::is_stickered(&t.g, bears));
    assert_eq!(t.pt(bears), (9, 9));
}

#[test]
fn a_melded_permanents_stickers_stay_with_the_card_its_owner_chooses() {
    cr!("123.5c");
    let setup = |t: &mut TestGame| -> ObjectId {
        limited(t, P0, vec![sheet("Stats", vec![pt_st(8, 2, 0), art()])]);
        let rats = t.battlefield(P0, "Graf Rats");
        let scavengers = t.battlefield(P0, "Midnight Scavengers");
        assert!(put(t, P0, rats, Some(StickerType::PowerToughness), 0));
        assert!(put(t, P0, scavengers, Some(StickerType::Art), 0));
        meld_host(t)
    };
    let card_in = |t: &TestGame, zone: &[ObjectId], name: &str| -> ObjectId {
        *zone.iter().find(|o| t.obj(**o).chars.name == name).unwrap()
    };
    // Chittering Host dies: P0 chooses Midnight Scavengers to keep both stickers.
    let mut t = TestGame::new(2);
    let host = setup(&mut t);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.g.move_object(host, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.g.recompute();
    let gy = t.g.players[P0.idx()].graveyard.clone();
    let rats = card_in(&t, &gy, "Graf Rats");
    let scavengers = card_in(&t, &gy, "Midnight Scavengers");
    assert!(!stickers::is_stickered(&t.g, rats));
    assert_eq!(stickers::stickers_on(&t.g, scavengers).len(), 2);
    assert_eq!(t.obj(scavengers).chars.power, Some(8));
    assert_eq!(t.obj(rats).chars.power, Some(2));
    // Choosing Graf Rats instead.
    let mut t = TestGame::new(2);
    let host = setup(&mut t);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.g.move_object(host, Zone::Exile, MoveCause::Effect, None);
    t.g.recompute();
    let ex = t.g.exile.clone();
    let rats = card_in(&t, &ex, "Graf Rats");
    let scavengers = card_in(&t, &ex, "Midnight Scavengers");
    assert_eq!(stickers::stickers_on(&t.g, rats).len(), 2);
    assert!(!stickers::is_stickered(&t.g, scavengers));
    // To a hidden zone, no card keeps them.
    let mut t = TestGame::new(2);
    let host = setup(&mut t);
    t.g.move_object(host, Zone::Hand(P0), MoveCause::Effect, None);
    t.g.recompute();
    let hand = t.g.players[P0.idx()].hand.clone();
    assert_eq!(hand.len(), 2);
    assert!(hand.iter().all(|o| !stickers::is_stickered(&t.g, *o)));
}
