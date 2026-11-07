//! Rulings batch P171 — token creation: token names (Sandsteppe War Riders), token
//! doublers (Anointed Procession, Parallel Lives, Exalted Sunborn), token copies (Battle
//! for Bretagard, Ocelot Pride, Worldwalker Helm, Specimen Collector).

use crate::r_p171_common::*;
use crate::r_s02_common::create_token;
use crate::r_s05_common::move_to;
use crate::r_s17_common::token_copy;
use crate::r_s19_common::add_lore;
use crate::r_s25_common::name_now;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn tokens_named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

fn token_count(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token())
        .count()
}

#[test]
fn sandsteppe_war_riders_myr_and_phyrexian_myr_tokens_have_different_names() {
    cr!("111.4", "701.39a");
    ruling!(
        "Sandsteppe War Riders",
        "A token’s name is its subtypes plus the word “Token” unless it’s copying another object or it was given a specific name by the effect that created it."
    );
    supported("Sandsteppe War Riders");
    supported("Master's Call");
    supported("Myr Sire");
    let mut t = TestGame::new(2);
    let riders = t.battlefield(P0, "Sandsteppe War Riders");
    // Master's Call: two Myr tokens. Myr Sire dies: a Phyrexian Myr token.
    cast_card(&mut t, P0, "Master's Call");
    t.resolve_all();
    let sire = t.battlefield(P0, "Myr Sire");
    move_to(&mut t, sire, Zone::Graveyard(P0));
    t.resolve_all();
    let toks: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is_token())
            .map(|o| o.id)
            .collect();
    assert_eq!(toks.len(), 3);
    let mut names: Vec<String> = toks.iter().map(|id| name_now(&t, *id)).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 2, "{names:?}");
    // Beginning of combat: bolster 2 (two differently named artifact tokens).
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    let counters: u32 = toks
        .iter()
        .chain(std::iter::once(&riders))
        .map(|id| t.counters(*id, "+1/+1"))
        .sum();
    assert_eq!(counters, 2);
}

#[test]
fn anointed_procession_tokens_enter_simultaneously_and_alike() {
    cr!("614.1a", "111.1");
    ruling!(
        "Anointed Procession",
        "All of the tokens enter the battlefield simultaneously. They’ll be created with the same name, color, type and subtype, abilities, power, toughness, and so on."
    );
    supported("Anointed Procession");
    supported("Woodland Champion");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anointed Procession");
    let champ = t.battlefield(P0, "Woodland Champion");
    // Raise the Alarm: two 1/1 white Soldier tokens → four, entering at once: Woodland
    // Champion ("Whenever one or more tokens you control enter, put that many +1/+1
    // counters on this creature") triggers once.
    cast_card(&mut t, P0, "Raise the Alarm");
    t.resolve();
    assert_eq!(triggers_from(&t, champ), 1);
    t.resolve_all();
    assert_eq!(t.counters(champ, "+1/+1"), 4);
    let soldiers = tokens_named(&t, P0, &name_now(&t, tokens_of_p0(&t)[0]));
    assert_eq!(soldiers.len(), 4);
    for s in &soldiers {
        let o = t.obj_now(*s);
        assert_eq!((o.power(), o.toughness()), (1, 1));
        assert!(o.chars.subtypes.iter().any(|x| x == "Soldier"));
        assert!(o.chars.colors.contains(types::Color::White));
    }
}

fn tokens_of_p0(t: &TestGame) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.is_token())
        .map(|o| o.id)
        .collect()
}

#[test]
fn anointed_procession_doubles_each_kind_of_token() {
    cr!("614.1a", "111.1");
    ruling!(
        "Anointed Procession",
        "If an effect creates more than one kind of token, it’ll create twice as many of each kind."
    );
    supported("Bestial Menace");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anointed Procession");
    cast_card(&mut t, P0, "Bestial Menace");
    t.resolve_all();
    for kind in ["Snake", "Wolf", "Elephant"] {
        let n =
            t.g.permanents()
                .filter(|o| o.is_token() && o.chars.subtypes.iter().any(|s| s == kind))
                .count();
        assert_eq!(n, 2, "{kind}");
    }
}

#[test]
fn anointed_procession_later_instructions_apply_to_all_the_tokens() {
    cr!("614.1a", "603.7a");
    ruling!(
        "Anointed Procession",
        "If the effect creating the tokens instructs you to do something with those tokens at a later time, like exiling them at the end of combat, you’ll do that for all the tokens."
    );
    supported("Elemental Appeal");
    // Elemental Appeal: "Create a 7/1 red Elemental creature token with trample and
    // haste. Exile it at the beginning of the next end step."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anointed Procession");
    cast_card(&mut t, P0, "Elemental Appeal");
    t.resolve_all();
    assert_eq!(token_count(&t, P0), 2);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(token_count(&t, P0), 0);
}

#[test]
fn doublers_copy_tapped_and_attacking_and_counters() {
    cr!("614.1a", "508.4");
    ruling!(
        "Parallel Lives",
        "Everything that is specified by the effect creating the original token or tokens will also be true about the additional token or tokens created by Parallel Lives's replacement effect."
    );
    ruling!(
        "Exalted Sunborn",
        "Everything that is specified by the effect creating the original token or tokens will also be true about the additional token or tokens created by Exalted Sunborn’s replacement effect."
    );
    supported("Parallel Lives");
    supported("Exalted Sunborn");
    supported("Hero of Bladehold");
    // Hero of Bladehold: "Whenever this creature attacks, create two 1/1 white Soldier
    // creature tokens that are tapped and attacking."
    for doubler in ["Parallel Lives", "Exalted Sunborn"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, doubler);
        let hero = t.battlefield(P0, "Hero of Bladehold");
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(hero, Entity::Player(P1))]),
        );
        t.advance_to(P0, Step::DeclareAttackers);
        t.settle();
        t.resolve_all();
        let toks = tokens_of_p0(&t);
        assert_eq!(toks.len(), 4, "{doubler}");
        let attacking = |id: ObjectId| {
            t.g.combat
                .as_ref()
                .is_some_and(|c| c.attackers.iter().any(|a| a.id == id))
        };
        for id in &toks {
            assert!(t.obj_now(*id).tapped, "{doubler}");
            assert!(attacking(*id), "{doubler}");
        }
    }
    // Exalted Sunborn: a token created with counters (Body of Research: "Create a 0/0
    // green and blue Fractal creature token. Put X +1/+1 counters on it, where X is the
    // number of cards in your library.") — the additional one gets them too.
    supported("Body of Research");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exalted Sunborn");
    let lib = t.library_size(P0) as u32;
    cast_card(&mut t, P0, "Body of Research");
    t.resolve_all();
    let toks = tokens_of_p0(&t);
    assert_eq!(toks.len(), 2);
    for id in toks {
        assert_eq!(t.counters(id, "+1/+1"), lib);
    }
}

#[test]
fn battle_for_bretagard_copies_only_copiable_values() {
    cr!("707.2", "714.2b");
    ruling!(
        "Battle for Bretagard",
        "Each token you create copies the original characteristics of the token it's copying as stated by the effect that created that token."
    );
    supported("Battle for Bretagard");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Battle for Bretagard");
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    let warrior = tokens_of_p0(&t)[0];
    // The Human Warrior token: tapped, a +1/+1 counter, Giant Growth.
    t.g.tap(warrior);
    t.g.add_counters(obj(warrior), types::counters::PLUS1, 1, None);
    cast_targeting(&mut t, P0, "Giant Growth", &[obj(warrior)]);
    t.resolve_all();
    assert_eq!(t.pt(warrior), (5, 5));
    // Chapter II: an Elf Warrior. Chapter III: copy both tokens (different names).
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    let before = tokens_of_p0(&t);
    let elf = *before.iter().find(|id| **id != warrior).unwrap();
    t.answer_choose(P0, &[obj(warrior), obj(elf)]);
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    let new: Vec<ObjectId> = tokens_of_p0(&t)
        .into_iter()
        .filter(|id| !before.contains(id))
        .collect();
    assert_eq!(new.len(), 2);
    let copy = *new
        .iter()
        .find(|id| name_now(&t, **id) == name_now(&t, warrior))
        .unwrap();
    assert!(!t.obj_now(copy).tapped);
    assert_eq!(t.counters(copy, "+1/+1"), 0);
    assert_eq!(t.pt(copy), (1, 1));
}

#[test]
fn copies_of_tokens_get_their_enters_abilities() {
    cr!("707.2", "603.6a");
    ruling!(
        "Battle for Bretagard",
        "Any enters-the-battlefield abilities of tokens you create will trigger when they enter the battlefield. Any \"As [this permanent] enters the battlefield\" or \"[This permanent] enters the battlefield with\" abilities of the tokens will also work."
    );
    ruling!(
        "Ocelot Pride",
        "Any enters-the-battlefield abilities of tokens you create will trigger when they enter the battlefield."
    );
    ruling!(
        "Worldwalker Helm",
        "Any enters-the-battlefield abilities of the copied token will trigger when the token enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the copied token will also work."
    );
    supported("Elvish Visionary");
    supported("Ocelot Pride");
    supported("Worldwalker Helm");
    // A token copy of Elvish Visionary ("When this creature enters, draw a card.").
    let setup = |t: &mut TestGame| -> ObjectId {
        let v = t.battlefield(P0, "Elvish Visionary");
        let tok = token_copy(t, P0, v)[0];
        t.resolve_all();
        tok
    };
    // Battle for Bretagard, chapter III.
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Battle for Bretagard");
    add_lore(&mut t, saga, 2);
    t.resolve_all();
    let tok = setup(&mut t);
    // And a token copy of Servant of the Scale ("This creature enters with a +1/+1
    // counter on it.").
    let servant = t.battlefield(P0, "Servant of the Scale");
    let stok = token_copy(&mut t, P0, servant)[0];
    assert_eq!(t.counters(stok, "+1/+1"), 1);
    let before = tokens_of_p0(&t);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(tok), obj(stok)]);
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    let scopy = tokens_of_p0(&t)
        .into_iter()
        .find(|id| !before.contains(id) && name_now(&t, *id) == "Servant of the Scale")
        .expect("a copy of the Servant token");
    assert_eq!(t.counters(scopy, "+1/+1"), 1);
    // Ocelot Pride with the city's blessing, at the end step of a turn P0 gained life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ocelot Pride");
    t.g.players[P0.idx()].has_citys_blessing = true;
    setup(&mut t);
    t.g.gain_life(P0, 1);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Worldwalker Helm: "{1}{U}, {T}: Create a token that's a copy of target artifact
    // token you control." A token copy of Ichor Wellspring ("When this artifact enters or
    // is put into a graveyard from the battlefield, draw a card.").
    let mut t = TestGame::new(2);
    let helm = t.battlefield(P0, "Worldwalker Helm");
    let well = t.battlefield(P0, "Ichor Wellspring");
    let tok = token_copy(&mut t, P0, well)
        .into_iter()
        .find(|id| name_now(&t, *id) == "Ichor Wellspring")
        .unwrap();
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.lands(P0, "Island", 2);
    t.activate(P0, helm, 0, &[obj(tok)]).expect("activate Helm");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // A token copy of Arcbound Worker (modular 1: "This enters with a +1/+1 counter on
    // it."): the Helm's copy enters with the counter too.
    let worker = t.battlefield(P0, "Arcbound Worker");
    let wtok = token_copy(&mut t, P0, worker)
        .into_iter()
        .find(|id| name_now(&t, *id) == "Arcbound Worker")
        .unwrap();
    t.g.untap(helm);
    let before = tokens_of_p0(&t);
    t.lands(P0, "Island", 2);
    t.activate(P0, helm, 0, &[obj(wtok)])
        .expect("activate Helm");
    t.resolve_all();
    let wcopy = tokens_of_p0(&t)
        .into_iter()
        .find(|id| !before.contains(id) && name_now(&t, *id) == "Arcbound Worker")
        .expect("a copy of the Worker token");
    assert_eq!(t.counters(wcopy, "+1/+1"), 1);
}

#[test]
fn worldwalker_helm_a_copied_x_cost_has_x_0() {
    cr!("707.2", "202.3e");
    ruling!(
        "Worldwalker Helm",
        "If the copied token has {X} in its mana cost, X is 0."
    );
    // Chalice of the Void ({X}{X}, "enters with X charge counters") cast for X = 2, and a
    // token copy of it: X is 0 for the copies, so they enter with no counters.
    let mut t = TestGame::new(2);
    let helm = t.battlefield(P0, "Worldwalker Helm");
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Chalice of the Void");
    t.cast(P0, card).x(2).go();
    t.resolve_all();
    let chalice = t.g.current(card);
    assert!(t.on_battlefield(chalice));
    assert_eq!(t.counters(chalice, "charge"), 2);
    let tok = token_copy(&mut t, P0, chalice)
        .into_iter()
        .find(|id| name_now(&t, *id) == "Chalice of the Void")
        .unwrap();
    assert_eq!(t.counters(tok, "charge"), 0);
    let before = tokens_of_p0(&t);
    t.lands(P0, "Island", 2);
    t.activate(P0, helm, 0, &[obj(tok)]).expect("activate Helm");
    t.resolve_all();
    let copy = tokens_of_p0(&t)
        .into_iter()
        .find(|id| !before.contains(id) && name_now(&t, *id) == "Chalice of the Void")
        .unwrap();
    assert_eq!(t.g.mana_value_of(copy), 0);
    assert_eq!(t.counters(copy, "charge"), 0);
}

#[test]
fn specimen_collector_can_copy_any_token_you_control() {
    cr!("707.2", "111.1");
    ruling!(
        "Specimen Collector",
        "For Specimen Collector's last ability, the target token can be any token you control, not necessarily one of the token creatures you created due to its first ability, and not necessarily even a creature."
    );
    supported("Specimen Collector");
    let mut t = TestGame::new(2);
    let collector = t.battlefield(P0, "Specimen Collector");
    let treasure = create_token(&mut t, P0, "Treasure");
    t.answer_targets(P0, &[obj(treasure)]);
    move_to(&mut t, collector, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, &name_now(&t, treasure)).len(), 2);
}
