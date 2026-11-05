//! Rulings batch P019 — effects that reveal or exile cards from a library until they find
//! enough of some kind and put them onto the battlefield (CR 701.20, 603.3b): Mass
//! Polymorph, Synthetic Destiny, Selvala's Stampede, Reality Scramble, Shifting Shadow,
//! Auspicious Starrix; and Eerie Ultimatum returning an Aura (CR 303.4f). Cards put onto
//! the battlefield together enter at the same time; triggers wait until the effect
//! finishes, then go on the stack in APNAP order.

use crate::r_p019_common::*;
use crate::r_p125_common::shuffled;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s06_common::{activate_containing, attached_to, give_control};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The controllers of the triggered abilities on the stack, bottom first.
fn trigger_controllers(t: &TestGame) -> Vec<PlayerId> {
    t.g.stack
        .iter()
        .filter(|id| {
            matches!(
                t.g.obj(**id).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Triggered { .. })
            )
        })
        .map(|id| t.g.obj(*id).controller)
        .collect()
}

#[test]
fn mass_polymorphs_creatures_enter_at_the_same_time() {
    cr!("701.20a", "603.6a");
    ruling!(
        "Mass Polymorph",
        "All creatures you put onto the battlefield with Mass Polymorph enter at the same time."
    );
    supported("Mass Polymorph");
    // "Exile all creatures you control, then reveal cards from the top of your library
    // until you reveal that many creature cards. Put all creature cards revealed this way
    // onto the battlefield, then shuffle the rest of the revealed cards into your
    // library." Hill Giant is revealed before Soul Warden ("Whenever another creature
    // enters, you gain 1 life."), but the Warden sees it enter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Hill Giant", "Sol Ring", "Soul Warden"]);
    cast_new(&mut t, P0, "Mass Polymorph", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Soul Warden").len(), 1);
    assert_eq!(t.life(P0), 21);
    assert_eq!(in_library(&t, P0, "Sol Ring"), 1);
}

#[test]
fn mass_polymorphs_triggers_wait_and_go_on_the_stack_in_apnap_order() {
    cr!("603.3b", "101.4");
    ruling!(
        "Mass Polymorph",
        "Any abilities that trigger during the resolution of Mass Polymorph (such as a creature's \"enters\" ability) will wait to be put onto the stack until Mass Polymorph finishes resolving. The player whose turn it is will put all of their triggered abilities on the stack in any order, then each other player in turn order will do the same. (The last ability put on the stack will be the first one that resolves.)"
    );
    supported("Elvish Visionary");
    // P0's Elvish Visionary ("When this creature enters, draw a card.") and P1's Soul
    // Warden (once for each creature) trigger: P0's go on the stack first.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Soul Warden");
    stack_library(&mut t, P0, &["Elvish Visionary", "Hill Giant"]);
    let polymorph = cast_new(&mut t, P0, "Mass Polymorph", &[]);
    t.resolve();
    assert!(!t.g.stack.contains(&polymorph));
    assert!(t.in_graveyard(P0, "Mass Polymorph"));
    assert_eq!(trigger_controllers(&t), vec![P0, P1, P1]);
    t.resolve_all();
    assert_eq!(t.life(P1), 22);
}

#[test]
fn revealing_for_more_creatures_than_the_library_has_reveals_all_of_it() {
    cr!("701.20a", "701.24a");
    ruling!(
        "Mass Polymorph",
        "If the number of creatures you exile is greater than the number of creature cards remaining in your library, you'll wind up revealing your entire library, putting all creature cards revealed that way onto the battlefield, then shuffling your library."
    );
    ruling!(
        "Mass Polymorph",
        "The creatures you exile with Mass Polymorph remain exiled for the rest of the game."
    );
    ruling!(
        "Synthetic Destiny",
        "If the number of creatures you exile is greater than the number of creature cards remaining in your library, you'll wind up revealing your entire library, putting all creature cards revealed that way onto the battlefield, then shuffling your library."
    );
    supported("Synthetic Destiny");
    // Three creatures exiled; the library holds one creature card (Hill Giant).
    let mut t = TestGame::new(2);
    let mine: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    stack_library(&mut t, P0, &["Sol Ring", "Hill Giant"]);
    let library = t.library_size(P0);
    cast_new(&mut t, P0, "Mass Polymorph", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.library_size(P0), library - 1);
    assert!(shuffled(&t, P0));
    // The exiled Bears stay exiled.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(mine.iter().all(|b| t.zone(*b) == Zone::Exile));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());

    // Synthetic Destiny: "Exile all creatures you control. At the beginning of the next
    // end step, reveal cards from the top of your library until you reveal that many
    // creature cards, put all creature cards revealed this way onto the battlefield, then
    // shuffle the rest of the revealed cards into your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Sol Ring", "Hill Giant"]);
    let library = t.library_size(P0);
    cast_new(&mut t, P0, "Synthetic Destiny", &[]);
    t.resolve_all();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.library_size(P0), library - 1);
    assert!(shuffled(&t, P0));
}

/// `p` votes for option `i` (Selvala's Stampede: 0 = wild, 1 = free).
fn vote(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn selvalas_stampede_wild_cards_enter_together_then_free_cards() {
    cr!("701.38a", "608.2c");
    ruling!(
        "Selvala's Stampede",
        "The creature cards revealed for the wild votes all enter the battlefield at the same time, followed by the permanent cards put onto the battlefield from your hand for the free votes."
    );
    supported("Selvala's Stampede");
    // "Council's dilemma — Starting with you, each player votes for wild or free. Reveal
    // cards from the top of your library until you reveal a creature card for each wild
    // vote. Put those creature cards onto the battlefield, then shuffle the rest into your
    // library. You may put a permanent card from your hand onto the battlefield for each
    // free vote."
    // Two wild votes: Hill Giant (revealed first) and Soul Warden enter together.
    let mut t = TestGame::new(2);
    vote(&mut t, P0, 0);
    vote(&mut t, P1, 0);
    stack_library(&mut t, P0, &["Hill Giant", "Soul Warden"]);
    cast_new(&mut t, P0, "Selvala's Stampede", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.life(P0), 21);
    // One of each: Soul Warden (wild) is on the battlefield as the Bears (free) enter.
    let mut t = TestGame::new(2);
    vote(&mut t, P0, 0);
    vote(&mut t, P1, 1);
    stack_library(&mut t, P0, &["Soul Warden"]);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Selvala's Stampede", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn selvalas_stampede_triggers_wait_and_go_on_the_stack_in_apnap_order() {
    cr!("603.3b", "101.4");
    ruling!(
        "Selvala's Stampede",
        "Any abilities that trigger because of any of those cards entering the battlefield are put onto the stack at the same time, after Selvala’s Stampede finishes resolving. You choose the order that abilities you control go on the stack, followed by each other player in turn order choosing the order for abilities they control. The last ability to be put on the stack will resolve first."
    );
    // P0's Elvish Visionary (wild) and P1's Soul Warden.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Soul Warden");
    vote(&mut t, P0, 0);
    vote(&mut t, P1, 1);
    stack_library(&mut t, P0, &["Elvish Visionary"]);
    let stampede = cast_new(&mut t, P0, "Selvala's Stampede", &[]);
    t.resolve();
    assert!(!t.g.stack.contains(&stampede));
    assert_eq!(trigger_controllers(&t), vec![P0, P1]);
}

/// P0 casts Reality Scramble ("Put target permanent you own on the bottom of your library.
/// Reveal cards from the top of your library until you reveal a card that shares a card
/// type with that permanent. Put that card onto the battlefield and the rest on the bottom
/// of your library in a random order.") targeting `target`, and it resolves.
fn scramble(t: &mut TestGame, target: ObjectId) {
    cast_new(t, P0, "Reality Scramble", &[Entity::Object(target)]);
    t.resolve_all();
}

#[test]
fn reality_scramble_stops_at_a_card_sharing_any_card_type() {
    cr!("701.20a");
    ruling!(
        "Reality Scramble",
        "You’ll stop as soon as you reveal a card that shares any card types with the target permanent. For example, if the target permanent is an artifact creature, you’ll stop as soon as you reveal a card that’s an artifact or a creature."
    );
    supported("Reality Scramble");
    // Ornithopter (artifact creature): Shock is passed over, Sol Ring (artifact) is put
    // onto the battlefield, the Bears stay in the library.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    stack_library(&mut t, P0, &["Shock", "Sol Ring", "Grizzly Bears"]);
    scramble(&mut t, thopter);
    assert_eq!(t.named_on_battlefield("Sol Ring").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(in_library(&t, P0, "Shock"), 1);
    assert_eq!(in_library(&t, P0, "Ornithopter"), 1);
}

#[test]
fn reality_scramble_on_a_token_with_no_matching_card_reveals_everything() {
    cr!("701.20a", "111.7");
    ruling!(
        "Reality Scramble",
        "In most cases, at least one card in your library (the card you put on the bottom of your library) will share a card type with the target permanent. However, in some unusual situations, no cards in your library share a card type with the permanent (perhaps because the permanent was a token or its types were changed). In that case, you’ll reveal your entire library, put nothing onto the battlefield, and put the cards back in a random order."
    );
    // A creature token; the library holds only a Sol Ring and fillers.
    let mut t = TestGame::new(2);
    let token = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    t.library_top(P0, "Sol Ring");
    let library = t.library_size(P0);
    let before: Vec<ObjectId> = t.g.battlefield.clone();
    scramble(&mut t, token);
    assert!(!t.on_battlefield(token));
    assert_eq!(t.library_size(P0), library);
    assert_eq!(in_library(&t, P0, "Sol Ring"), 1);
    assert!(t
        .g
        .battlefield
        .iter()
        .all(|o| before.contains(o) || t.obj(*o).is(CardType::Land)));
}

#[test]
fn reality_scramble_targets_a_permanent_you_own_that_another_player_controls() {
    cr!("115.1", "108.3");
    ruling!(
        "Reality Scramble",
        "Reality Scramble can target any permanent you own, including those another player controls. A token’s owner is the player who created it."
    );
    // P0's Ornithopter under P1's control goes to the bottom of P0's library; P0 reveals
    // Sol Ring and gets it.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    give_control(&mut t, thopter, P1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P0, &["Sol Ring"]);
    let from = t.asked().len();
    scramble(&mut t, thopter);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&Entity::Object(thopter)));
    assert!(!offered[0].contains(&Entity::Object(bears)));
    assert_eq!(in_library(&t, P0, "Ornithopter"), 1);
    let rings = t.named_on_battlefield("Sol Ring");
    assert_eq!(rings.len(), 1);
    assert_eq!(t.obj_now(rings[0]).controller, P0);
}

#[test]
fn reality_scramble_uses_the_targets_last_known_card_types() {
    cr!("701.20a", "608.2h", "113.7a");
    ruling!(
        "Reality Scramble",
        "The target permanent’s last known existence on the battlefield is checked to determine its types."
    );
    supported("Mutavault");
    // Mutavault ("{1}: This land becomes a 2/2 creature with all creature types until end
    // of turn. It's still a land."), animated: a land creature. In the library it's just
    // a land, but the Bears (revealed first) share a type with what it was.
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Mutavault");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, vault, "becomes a 2/2")
        .unwrap()
        .unwrap();
    t.resolve_all();
    assert!(t.obj_now(vault).is(CardType::Creature));
    stack_library(&mut t, P0, &["Grizzly Bears", "Forest"]);
    scramble(&mut t, vault);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(in_library(&t, P0, "Forest"), 1);
}

/// Shifting Shadow ("Enchanted creature has haste and \"At the beginning of your upkeep,
/// destroy this creature. Reveal cards from the top of your library until you reveal a
/// creature card. Put that card onto the battlefield and attach Shifting Shadow to it,
/// then put all other cards revealed this way on the bottom of your library in a random
/// order.\"") on P0's Bears; P0's next upkeep trigger resolves. Returns the Shadow.
fn shifting_shadow_upkeep(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shadow = crate::r_s06_common::attach_new(t, P0, "Shifting Shadow", bears);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    shadow
}

#[test]
fn shifting_shadow_that_cant_be_attached_is_put_into_the_graveyard() {
    cr!("702.16c", "704.5m");
    ruling!(
        "Shifting Shadow",
        "If Shifting Shadow can't become attached to the creature you put onto the battlefield, most likely because that creature has protection from red, it becomes attached to nothing. It will be put into its owner's graveyard as a state-based action."
    );
    supported("Shifting Shadow");
    supported("Kor Firewalker");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Kor Firewalker"]);
    let shadow = shifting_shadow_upkeep(&mut t);
    assert_eq!(t.named_on_battlefield("Kor Firewalker").len(), 1);
    assert!(t.in_graveyard(P0, "Shifting Shadow"));
    assert!(!t.on_battlefield(shadow));
    // A creature without protection: the Shadow moves to it.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Hill Giant"]);
    let shadow = shifting_shadow_upkeep(&mut t);
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(attached_to(&t, shadow), Some(Entity::Object(giant)));
}

#[test]
fn shifting_shadow_with_no_creature_card_goes_to_the_graveyard() {
    cr!("701.20a", "704.5m");
    ruling!(
        "Shifting Shadow",
        "If you don't reveal a creature card this way, you'll just reveal and randomize your library and Shifting Shadow becomes attached to nothing. It will be put into its owner's graveyard as a state-based action."
    );
    let mut t = TestGame::new(2);
    t.library_top(P0, "Sol Ring");
    let shadow = shifting_shadow_upkeep(&mut t);
    assert!(!t.on_battlefield(shadow));
    assert!(t.in_graveyard(P0, "Shifting Shadow"));
    assert_eq!(in_library(&t, P0, "Sol Ring"), 1);
    assert!(crate::r_s01_common::creatures(&t, P0).is_empty());
}

/// P0 casts Auspicious Starrix for its mutate cost onto P0's Grizzly Bears (on top);
/// "Whenever this creature mutates, exile cards from the top of your library until you
/// exile X permanent cards, where X is the number of times this creature has mutated.
/// Put those permanent cards onto the battlefield." resolves (X = 1).
fn starrix_mutates(t: &mut TestGame) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 5);
    let starrix = t.hand(P0, "Auspicious Starrix");
    t.cast(P0, starrix)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
}

#[test]
fn auspicious_starrix_puts_lands_and_auras_onto_the_battlefield_and_leaves_the_rest_exiled() {
    cr!("305.4", "303.4f");
    ruling!(
        "Auspicious Starrix",
        "Auspicious Starrix’s effect doesn’t count as playing lands. It can put land cards onto the battlefield even if it’s not your turn or if you’ve already played your land for the turn."
    );
    ruling!(
        "Auspicious Starrix",
        "Any cards that you exile but don’t put on the battlefield for any reason (for example, because the card is an instant card or an Aura that can’t enchant anything) remain in exile."
    );
    ruling!(
        "Auspicious Starrix",
        "If an Aura is put onto the battlefield without being cast, the Aura’s controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof, for example), but the Aura’s enchant ability restricts what it can be attached to. If the Aura can’t legally be attached to anything, it remains in its current zone."
    );
    supported("Auspicious Starrix");
    supported("Gladecover Scout");
    // P0 already played a land this turn. The Starrix exiles Shock (stays exiled) and
    // Forest (enters, not as a land play).
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].lands_played_this_turn = 1;
    let cards = stack_library(&mut t, P0, &["Shock", "Forest", "Hill Giant"]);
    starrix_mutates(&mut t);
    assert_eq!(t.zone(cards[1]), Zone::Battlefield);
    assert!(t.in_exile("Shock"));
    assert_eq!(in_library(&t, P0, "Hill Giant"), 1);
    assert_eq!(t.g.players[P0.idx()].lands_played_this_turn, 1);
    // Pacifism, exiled, is put onto the battlefield enchanting P1's hexproof Gladecover
    // Scout.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    stack_library(&mut t, P0, &["Pacifism"]);
    t.answer_choose(P0, &[Entity::Object(scout)]);
    starrix_mutates(&mut t);
    let pacifism = t.named_on_battlefield("Pacifism");
    assert_eq!(pacifism.len(), 1);
    assert_eq!(attached_to(&t, pacifism[0]), Some(Entity::Object(scout)));
}

#[test]
fn auspicious_starrix_with_too_few_permanent_cards_exiles_the_whole_library() {
    cr!("608.2c");
    ruling!(
        "Auspicious Starrix",
        "If there are fewer than X permanent cards in your library, you exile your entire library and put the permanent cards in it onto the battlefield."
    );
    // No permanent cards in the library (Shock and fillers).
    let mut t = TestGame::new(2);
    t.library_top(P0, "Shock");
    starrix_mutates(&mut t);
    assert_eq!(t.library_size(P0), 0);
    assert!(t.in_exile("Shock"));
}

#[test]
fn eerie_ultimatum_aura_chooses_what_to_enchant_or_stays_in_the_graveyard() {
    cr!("303.4f", "702.11b");
    ruling!(
        "Eerie Ultimatum",
        "If an Aura is put onto the battlefield without being cast, the Aura’s controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof, for example), but the Aura’s enchant ability restricts what it can be attached to. If the Aura can’t legally be attached to anything, it remains in its current zone."
    );
    supported("Eerie Ultimatum");
    // "Return any number of permanent cards with different names from your graveyard to
    // the battlefield." Pacifism enchants P1's hexproof Gladecover Scout.
    let ultimatum = |t: &mut TestGame, cards: &[ObjectId], enchant: Option<ObjectId>| {
        t.lands(P0, "Plains", 2);
        t.lands(P0, "Swamp", 3);
        t.lands(P0, "Forest", 2);
        let es: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(P0, &es);
        if let Some(e) = enchant {
            t.answer_choose(P0, &[Entity::Object(e)]);
        }
        let card = t.hand(P0, "Eerie Ultimatum");
        t.cast(P0, card).go();
        t.resolve_all();
    };
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let pacifism = t.graveyard(P0, "Pacifism");
    ultimatum(&mut t, &[pacifism], Some(scout));
    let on = t.named_on_battlefield("Pacifism");
    assert_eq!(on.len(), 1);
    assert_eq!(attached_to(&t, on[0]), Some(Entity::Object(scout)));
    // With no creature on the battlefield, Pacifism stays in the graveyard.
    let mut t = TestGame::new(2);
    let pacifism = t.graveyard(P0, "Pacifism");
    ultimatum(&mut t, &[pacifism], None);
    assert!(t.named_on_battlefield("Pacifism").is_empty());
    assert!(t.in_graveyard(P0, "Pacifism"));
}
