//! Rulings batch P172 — cards that care about tokens: token-doubling and "those tokens
//! plus" replacement effects (CR 614.1a, 616.1), copies of tokens (CR 707.2, 111.10),
//! populate (CR 701.36), creature tokens leaving the battlefield, and token names
//! (CR 111.4).

use crate::r_p017_common::choice_candidates;
use crate::r_p076_common::mana;
use crate::r_s01_common::{supported, tokens};
use crate::r_s02_common::create_token;
use crate::r_s04_common::run_with;
use crate::r_s05_common::{enter, move_to};
use crate::r_s08_common::{legal_cast_methods, mana_value};
use crate::r_s17_common::token_copy;
use crate::r_s19_common::gain_level;
use mtg_engine::ability::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `p` creates `count` predefined tokens (`name`, e.g. "Treasure"), tapped if `tapped`,
/// as a resolving effect would.
fn create_tapped(t: &mut TestGame, p: PlayerId, name: &str, tapped: bool) {
    run_with(
        t,
        p,
        Effect::CreateToken {
            spec: mtg_engine::tokens::predefined(name).unwrap(),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped,
            attacking: false,
        },
        &[],
    );
}

fn named_tokens(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    tokens(t, p)
        .into_iter()
        .filter(|id| {
            let n = &t.obj(*id).chars.name;
            n == name || *n == format!("{name} Token")
        })
        .collect()
}

#[test]
fn token_doublers_are_cumulative() {
    cr!("614.1a", "616.1");
    ruling!(
        "Anointed Procession",
        "If you control two Anointed Processions, then the number of tokens created is four times the original number. If you control three, then the number of tokens created is eight times the original number, and so on."
    );
    ruling!(
        "Parallel Lives",
        "If you control two Parallel Lives, then the number of tokens created is four times the original number. If you control three, then the number of tokens created is eight times the original number, and so on."
    );
    for name in ["Anointed Procession", "Parallel Lives"] {
        supported(name);
        for (n, expect) in [(1, 2), (2, 4), (3, 8)] {
            let mut t = TestGame::new(2);
            for _ in 0..n {
                t.battlefield(P0, name);
            }
            create_token(&mut t, P0, "Soldier");
            assert_eq!(tokens(&t, P0).len(), expect, "{n} {name}");
        }
    }
}

#[test]
fn doubled_tokens_apply_their_as_enters_abilities_individually() {
    cr!("614.12", "707.9", "702.128a");
    ruling!(
        "Anointed Procession",
        "If the token you create has any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities, first determine how many tokens are being created, then apply those abilities individually for each one."
    );
    supported("Vizier of Many Faces");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anointed Procession");
    t.lands(P0, "Island", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let vizier = t.graveyard(P0, "Vizier of Many Faces");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, vizier, 0, &[]).unwrap();
    t.resolve_all();
    let mut names: Vec<String> = tokens(&t, P0)
        .iter()
        .map(|id| t.obj(*id).chars.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Hill Giant"]);
}

#[test]
fn seasoned_warrenguard_doesnt_recheck_tokens_on_resolution() {
    cr!("603.2", "508.1m");
    ruling!(
        "Seasoned Warrenguard",
        "If you controlled at least one token when you declared Seasoned Warrenguard as an attacker, it doesn’t matter whether or not you control any tokens as its ability resolves."
    );
    supported("Seasoned Warrenguard");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Seasoned Warrenguard");
    let tok = create_token(&mut t, P0, "Soldier");
    crate::r_s01_common::attack_with(&mut t, &[(guard, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, tok, Zone::Exile);
    assert!(tokens(&t, P0).is_empty());
    t.resolve_all();
    assert_eq!(t.pt(guard), (3, 2));
    // Without a token, it doesn't trigger.
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Seasoned Warrenguard");
    crate::r_s01_common::attack_with(&mut t, &[(guard, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn mutagen_man_reduces_an_artifact_tokens_ability_to_zero() {
    cr!("601.2f", "602.2b");
    ruling!(
        "Mutagen Man, Living Ooze",
        "Mutagen Man's second ability can cause the mana component of the cost to activate an ability of an artifact token to be {0}."
    );
    supported("Mutagen Man, Living Ooze");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let man = t.battlefield(P0, "Mutagen Man, Living Ooze");
    let mutagen = create_token(&mut t, P0, "Mutagen");
    t.activate(P0, mutagen, 0, &[obj(man)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(man, "+1/+1"), 1);
    assert!(!t.on_battlefield(mutagen));
}

#[test]
fn additional_tokens_share_how_the_tokens_are_created_but_not_their_abilities() {
    cr!("614.1a", "111.1");
    ruling!(
        "Quina, Qu Gourmet",
        "The additional Frog token won't have any abilities the other tokens were created with. Anything else specified in the effect creating the token (such as tapped or \"Exile that token at end of combat\") applies to both the original tokens and the Frog."
    );
    ruling!(
        "Worldwalker Helm",
        "The additional Map token won't have any abilities the other tokens were created with. Anything else specified in the effect creating the token (such as tapped or \"Exile that token at end of combat\") applies to both the original tokens and the Map."
    );
    for (card, extra, abilities) in [("Quina, Qu Gourmet", "Frog Token", 0), ("Worldwalker Helm", "Map", 1)] {
        supported(card);
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        create_tapped(&mut t, P0, "Treasure", true);
        let treasure = named_tokens(&t, P0, "Treasure");
        let more = named_tokens(&t, P0, extra);
        assert_eq!((treasure.len(), more.len()), (1, 1), "{card}");
        assert!(t.obj(treasure[0]).tapped && t.obj(more[0]).tapped, "{card}");
        let o = t.obj(more[0]);
        assert_eq!(o.chars.abilities.len(), abilities, "{card}");
        assert!(o
            .chars
            .abilities
            .iter()
            .all(|a| !a.text.contains("Add one mana")));
    }
}

#[test]
fn worldwalker_helm_applies_to_every_artifact_token() {
    cr!("614.1a");
    ruling!(
        "Worldwalker Helm",
        "Worldwaker Helm's first ability applies to all kinds of artifact tokens that its controller creates, not just Map tokens."
    );
    for (token, maps) in [("Food", 1), ("Clue", 1), ("Soldier", 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Worldwalker Helm");
        create_token(&mut t, P0, token);
        assert_eq!(named_tokens(&t, P0, "Map").len(), maps, "{token}");
    }
}

#[test]
fn worldwalker_helm_copies_only_the_tokens_copiable_values() {
    cr!("707.2", "111.10");
    ruling!(
        "Worldwalker Helm",
        "The new token created by Worldwaker Helm's last ability copies the original characteristics of the target token as stated by the effect that created the target token and nothing else"
    );
    let mut t = TestGame::new(2);
    let helm = t.battlefield(P0, "Worldwalker Helm");
    create_tapped(&mut t, P0, "Clue", true);
    let clue = named_tokens(&t, P0, "Clue")[0];
    t.g.add_counters(obj(clue), "+1/+1", 2, None);
    t.lands(P0, "Island", 2);
    t.activate(P0, helm, 0, &[obj(clue)]).unwrap();
    t.resolve_all();
    let clues = named_tokens(&t, P0, "Clue");
    assert_eq!(clues.len(), 2);
    let copy = *clues.iter().find(|c| **c != clue).unwrap();
    assert!(!t.obj(copy).tapped);
    assert_eq!(t.counters(copy, "+1/+1"), 0);
    assert_eq!(t.obj(copy).chars.abilities.len(), 1);
}

#[test]
fn you_must_create_the_tokens_but_neednt_control_the_effect() {
    cr!("614.1a", "111.2");
    ruling!(
        "Quina, Qu Gourmet",
        "You don't need to control the spell or ability that creates the tokens, but you do have to be the one creating the tokens for Quina's first ability to apply."
    );
    ruling!(
        "Worldwalker Helm",
        "You don't need to control the spell or ability that creates the tokens, but you do have to be the one creating the tokens for Worldwaker Helm's first ability to apply."
    );
    supported("Beast Within");
    supported("Fateful Absence");
    // P1's Beast Within destroys P0's permanent: P0 creates the Beast, and a Frog.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quina, Qu Gourmet");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bw = t.hand(P1, "Beast Within");
    mana(&mut t, P1, ManaType::G, 3);
    t.cast(P1, bw).target(bears).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(named_tokens(&t, P0, "Frog Token").len(), 1);
    // P0's Beast Within on P1's permanent: P1 creates the Beast; no Frog.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quina, Qu Gourmet");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bw = t.hand(P0, "Beast Within");
    mana(&mut t, P0, ManaType::G, 3);
    t.cast(P0, bw).target(bears).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 0);
    assert_eq!(tokens(&t, P1).len(), 1);
    // P1's Fateful Absence: P0 investigates, creating a Clue and a Map.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Worldwalker Helm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fa = t.hand(P1, "Fateful Absence");
    mana(&mut t, P1, ManaType::W, 2);
    t.cast(P1, fa).target(bears).go();
    t.resolve_all();
    assert_eq!(named_tokens(&t, P0, "Clue").len(), 1);
    assert_eq!(named_tokens(&t, P0, "Map").len(), 1);
    // P0's Fateful Absence: P1 investigates; no Map.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Worldwalker Helm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let fa = t.hand(P0, "Fateful Absence");
    mana(&mut t, P0, ManaType::W, 2);
    t.cast(P0, fa).target(bears).go();
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(named_tokens(&t, P1, "Clue").len(), 1);
}

#[test]
fn lich_knights_conquest_sacrifices_on_resolution_and_maybe_nothing() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Lich-Knights' Conquest",
        "You can choose to sacrifice zero permanents."
    );
    ruling!(
        "Lich-Knights' Conquest",
        "You sacrifice the artifacts, enchantments, and/or tokens as part of the resolution of Lich-Knights' Conquest. It isn't an additional cost. If Lich-Knights' Conquest is countered, you won't sacrifice anything."
    );
    supported("Lich-Knights' Conquest");
    // Zero.
    let mut t = TestGame::new(2);
    let treasure = create_token(&mut t, P0, "Treasure");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let lkc = t.hand(P0, "Lich-Knights' Conquest");
    mana(&mut t, P0, ManaType::B, 5);
    t.cast(P0, lkc).go();
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(treasure));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    // Not a cost: nothing is sacrificed as it's cast, and nothing if it's countered.
    let mut t = TestGame::new(2);
    let treasure = create_token(&mut t, P0, "Treasure");
    t.graveyard(P0, "Grizzly Bears");
    let lkc = t.hand(P0, "Lich-Knights' Conquest");
    mana(&mut t, P0, ManaType::B, 5);
    let spell = t.cast(P0, lkc).go();
    assert!(t.on_battlefield(treasure));
    let cancel = t.hand(P1, "Cancel");
    mana(&mut t, P1, ManaType::U, 3);
    t.cast(P1, cancel).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lich-Knights' Conquest"));
    assert!(t.on_battlefield(treasure));
    // Resolving: the chosen token is sacrificed and that many creatures return.
    let mut t = TestGame::new(2);
    let treasure = create_token(&mut t, P0, "Treasure");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let lkc = t.hand(P0, "Lich-Knights' Conquest");
    mana(&mut t, P0, ManaType::B, 5);
    t.cast(P0, lkc).go();
    t.answer_choose(P0, &[obj(treasure)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(treasure));
    assert!(t.on_battlefield(t.g.current(bears)));
}

#[test]
fn phantom_general_pumps_a_token_copy_but_not_the_original() {
    cr!("111.10", "707.2");
    ruling!(
        "Phantom General",
        "A creature token that's a copy of a nontoken creature is still a token. It will get +1/+1, but the original nontoken creature will not."
    );
    supported("Phantom General");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phantom General");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let copy = token_copy(&mut t, P0, bears);
    assert_eq!(copy.len(), 1);
    assert_eq!(t.pt(copy[0]), (3, 3));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn an_adventurer_card_is_a_permanent_card_outside_the_stack() {
    cr!("715.4", "715.3");
    ruling!(
        "Promise of Aclazotz // Foul Rebirth",
        "An adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure."
    );
    supported("Promise of Aclazotz // Foul Rebirth");
    supported("Archaeomancer");
    let mut t = TestGame::new(2);
    let promise = t.graveyard(P0, "Promise of Aclazotz // Foul Rebirth");
    assert_eq!(mana_value(&t, promise), 2);
    assert!(t.obj(promise).is(CardType::Enchantment));
    assert!(!t.obj(promise).is(CardType::Sorcery));
    // Archaeomancer's "target instant or sorcery card" finds nothing.
    enter(&mut t, P0, "Archaeomancer");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(promise), Zone::Graveyard(P0));
}

#[test]
fn an_adventure_cast_from_the_library_uses_its_alternative_characteristics() {
    cr!("715.3a", "715.3b");
    ruling!(
        "Promise of Aclazotz // Foul Rebirth",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it's legal to cast that spell."
    );
    supported("Johann, Apprentice Sorcerer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Johann, Apprentice Sorcerer");
    t.battlefield(P0, "Grizzly Bears");
    let promise = t.library_top(P0, "Promise of Aclazotz // Foul Rebirth");
    t.lands(P0, "Swamp", 3);
    let methods = legal_cast_methods(&mut t, P0, promise);
    assert!(methods.iter().any(|m| matches!(m, CastMethod::Half(_))));
    assert!(!methods.contains(&CastMethod::Normal));
}

#[test]
fn full_flowering_populates_one_at_a_time_and_triggers_wait() {
    cr!("701.36a", "603.3");
    ruling!(
        "Full Flowering",
        "Any abilities that trigger while you're populating won't be put onto the stack until after you've finished populating X times."
    );
    ruling!("Full Flowering", "The tokens enter the battlefield one at a time.");
    supported("Full Flowering");
    supported("Daxos, Blessed by the Sun");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Daxos, Blessed by the Sun");
    create_token(&mut t, P0, "Spirit");
    create_token(&mut t, P0, "Plant");
    t.resolve_all();
    let ff = t.hand(P0, "Full Flowering");
    mana(&mut t, P0, ManaType::G, 5);
    t.cast(P0, ff).x(2).go();
    t.settle();
    let from = t.asked().len();
    let life = t.life(P0);
    t.resolve();
    // The second populate can copy the token the first one created.
    let lens: Vec<usize> = choice_candidates(&t, from).iter().map(|c| c.len()).collect();
    assert_eq!(lens, vec![2, 3]);
    assert_eq!(tokens(&t, P0).len(), 4);
    // Both Daxos triggers wait until it's done.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.life(P0), life);
    t.resolve_all();
    assert_eq!(t.life(P0), life + 2);
}

fn drover_with_counter(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let drover = t.battlefield(p, "Twilight Drover");
    t.g.add_counters(obj(drover), "+1/+1", 1, None);
    drover
}

#[test]
fn twilight_drover_counts_any_creature_token_leaving_to_anywhere() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Twilight Drover",
        "If Spirit tokens created by Twilight Drover leave the battlefield, they will trigger its first ability."
    );
    ruling!(
        "Twilight Drover",
        "The first ability only cares that creature tokens leave the battlefield. It doesn't matter where they went."
    );
    supported("Twilight Drover");
    let mut t = TestGame::new(2);
    let drover = drover_with_counter(&mut t, P0);
    mana(&mut t, P0, ManaType::W, 3);
    t.activate(P0, drover, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(drover, "+1/+1"), 0);
    let spirits = tokens(&t, P0);
    assert_eq!(spirits.len(), 2);
    move_to(&mut t, spirits[0], Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.counters(drover, "+1/+1"), 1);
    move_to(&mut t, spirits[1], Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.counters(drover, "+1/+1"), 2);
    // An opponent's creature token exiled; a noncreature token doesn't count.
    let tok = create_token(&mut t, P1, "Goblin");
    move_to(&mut t, tok, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.counters(drover, "+1/+1"), 3);
    let treasure = create_token(&mut t, P0, "Treasure");
    move_to(&mut t, treasure, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.counters(drover, "+1/+1"), 3);
}

#[test]
fn twilight_drover_triggers_for_each_token_of_a_player_leaving_the_game() {
    cr!("800.4a", "603.6c");
    ruling!(
        "Twilight Drover",
        "If a player leaves a multiplayer game, all creature tokens that player owns also leave the game. Twilight Drover's ability will trigger once per token."
    );
    let mut t = TestGame::new(3);
    let drover = t.battlefield(P0, "Twilight Drover");
    create_token(&mut t, P1, "Goblin");
    create_token(&mut t, P1, "Goblin");
    t.g.player_loses(P1);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(drover, "+1/+1"), 2);
}

#[test]
fn twilight_drover_cant_make_tokens_leave_itself() {
    cr!("113.3");
    ruling!(
        "Twilight Drover",
        "Note that Twilight Drover doesn't have any way to sacrifice tokens or otherwise cause them to leave the battlefield."
    );
    let mut t = TestGame::new(2);
    let drover = drover_with_counter(&mut t, P0);
    create_token(&mut t, P0, "Goblin");
    mana(&mut t, P0, ManaType::W, 3);
    t.activate(P0, drover, 0, &[]).unwrap();
    t.resolve_all();
    // Its only activated ability makes tokens; none left.
    assert_eq!(tokens(&t, P0).len(), 3);
    assert_eq!(t.counters(drover, "+1/+1"), 0);
}

#[test]
fn caretakers_talent_copies_the_tokens_original_characteristics() {
    cr!("707.2", "716.2a");
    ruling!(
        "Caretaker's Talent",
        "The new token created by the second class ability copies the characteristics of the original token as stated by the effect that created the original token."
    );
    supported("Caretaker's Talent");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ct = t.battlefield(P0, "Caretaker's Talent");
    let spirit = create_token(&mut t, P0, "Spirit");
    t.resolve_all();
    t.g.add_counters(obj(spirit), "+1/+1", 2, None);
    t.g.tap(spirit);
    mana(&mut t, P0, ManaType::W, 1);
    gain_level(&mut t, P0, ct, 2).unwrap();
    t.answer_targets(P0, &[obj(spirit)]);
    t.resolve_all();
    let spirits = named_tokens(&t, P0, "Spirit");
    assert_eq!(spirits.len(), 2);
    let copy = *spirits.iter().find(|s| **s != spirit).unwrap();
    assert_eq!(t.pt(copy), (1, 1));
    assert!(!t.obj(copy).tapped);
    assert_eq!(t.counters(copy, "+1/+1"), 0);
}

#[test]
fn audience_with_trostani_counts_differently_named_tokens() {
    cr!("111.4", "201.2");
    ruling!(
        "Audience with Trostani",
        "To determine the number of differently named creature tokens you control, count each creature token you control once, but only if its English name isn't exactly the same as another creature token you've already counted this way."
    );
    ruling!(
        "Audience with Trostani",
        "Unless a token is a copy of another permanent or was explicitly given a name by the effect that created it, its name is the subtypes it was given when it was created plus the word \"Token.\""
    );
    supported("Audience with Trostani");
    let mut t = TestGame::new(2);
    create_token(&mut t, P0, "Spirit");
    create_token(&mut t, P0, "Spirit");
    create_token(&mut t, P0, "Goblin");
    // A noncreature token isn't counted.
    create_token(&mut t, P0, "Treasure");
    let a = t.hand(P0, "Audience with Trostani");
    mana(&mut t, P0, ManaType::G, 3);
    t.cast(P0, a).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(named_tokens(&t, P0, "Plant Token").len(), 1);
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn combine_chrysalis_can_sacrifice_a_noncreature_token() {
    cr!("602.2", "118.3");
    ruling!(
        "Combine Chrysalis",
        "You can sacrifice any token you control to activate the last ability, not just creature tokens."
    );
    supported("Combine Chrysalis");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cc = t.battlefield(P0, "Combine Chrysalis");
    let treasure = create_token(&mut t, P0, "Treasure");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::U, 2);
    t.activate(P0, cc, 0, &[]).unwrap();
    assert!(!t.on_battlefield(treasure));
    t.resolve_all();
    assert_eq!(named_tokens(&t, P0, "Beast Token").len(), 1);
}

