//! Rulings batch S31 — casting cards from exile (and a graveyard) during the resolution
//! of an ability (CR 608.2g) without paying their mana costs (CR 118.9): the order of
//! several casts, what can be targeted, the characteristics of the cards in exile
//! (CR 712.8a, 709.4), {X} being 0 for a copy of an exiled card (CR 107.3b, 202.3e), and
//! a replacement that applies only to the spell's way to the graveyard (CR 614.1a).

use crate::r_s01_common::{attack_with, stack_library, supported};
use crate::r_s04_common::add_mana;
use crate::r_s22_common::choose_names_when_offered;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn etali_casts_the_exiled_spells_in_the_order_its_controller_chooses() {
    cr!("608.2g", "118.9", "601.2c", "115.1");
    ruling!(
        "Etali, Primal Storm",
        "If you cast more than one of the exiled cards, you choose the order in which to cast them. A spell you cast this way can be the target of a later spell you cast this way."
    );
    supported("Etali, Primal Storm");
    supported("Twincast");
    // "Whenever Etali attacks, exile the top card of each player's library, then you may
    // cast any number of spells from among those cards without paying their mana costs."
    // Lightning Strike first, then Twincast copying it ("Copy target instant or sorcery
    // spell. You may choose new targets for the copy.").
    let mut t = TestGame::new(2);
    let etali = t.battlefield(P0, "Etali, Primal Storm");
    t.library_top(P0, "Lightning Strike");
    t.library_top(P1, "Twincast");
    choose_names_when_offered(&mut t, P0, &["Lightning Strike", "Twincast"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(etali, Entity::Player(P1))]);
    t.resolve_all();
    // Lightning Strike and its copy each dealt 3 damage.
    assert_eq!(t.life(P1), 14);
    assert!(t.in_graveyard(P0, "Lightning Strike"));
    // Twincast is P1's card: it goes to P1's graveyard.
    assert!(t.in_graveyard(P1, "Twincast"));
}

#[test]
fn etalis_creature_spell_cant_be_enchanted_by_an_aura_cast_with_it() {
    cr!("608.2g", "303.4a", "115.1");
    ruling!(
        "Etali, Primal Storm",
        "However, permanent spells cast this way won't resolve until you're done casting spells, so the permanents they become can't be the target of spells cast this way."
    );
    supported("Pacifism");
    // Grizzly Bears and Pacifism ("Enchant creature") exiled, the Bears cast first: it's
    // still a spell on the stack as Pacifism is cast, so Pacifism can't target it (only
    // Etali).
    let mut t = TestGame::new(2);
    let etali = t.battlefield(P0, "Etali, Primal Storm");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Pacifism");
    choose_names_when_offered(&mut t, P0, &["Grizzly Bears", "Pacifism"]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(etali, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    let bears_now = t.g.current(bears);
    let enchanted =
        t.g.battlefield
            .iter()
            .any(|a| t.g.obj(*a).attached_to == Some(Entity::Object(bears_now)));
    assert!(!enchanted, "Pacifism couldn't target the Bears spell");
    // Its only legal target was Etali: it resolved before the Bears and enchants Etali.
    let pacifism = t.named_on_battlefield("Pacifism");
    assert_eq!(pacifism.len(), 1, "Pacifism resolved");
    assert_eq!(
        t.obj_now(pacifism[0]).attached_to,
        Some(Entity::Object(etali))
    );
}

#[test]
fn goblin_dark_dwellers_spell_returned_to_hand_isnt_exiled_later() {
    cr!("608.2g", "614.1a", "702.27a", "400.7");
    ruling!(
        "Goblin Dark-Dwellers",
        "If an instant or sorcery card you cast this way goes to a zone other than exile or a graveyard, perhaps because one of its abilities says to put it into its owner's hand, it won't be exiled. This is true even if the card would be put into a graveyard later that turn."
    );
    supported("Goblin Dark-Dwellers");
    supported("Capsize");
    // "When this creature enters, you may cast target instant or sorcery card with mana
    // value 3 or less from your graveyard without paying its mana cost. If that spell would
    // be put into your graveyard, exile it instead." Capsize {1}{U}{U}, buyback {3}:
    // "Return target permanent to its owner's hand."
    let mut t = TestGame::new(2);
    let capsize = t.graveyard(P0, "Capsize");
    let giant = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::C, 3);
    t.answer_targets(P0, &[Entity::Object(capsize)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Goblin Dark-Dwellers");
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
    // Buyback put Capsize into P0's hand instead.
    assert_eq!(t.zone(capsize), Zone::Hand(P0));
    // Discarded later this turn, it goes to the graveyard.
    let c = t.g.current(capsize);
    t.g.discard(P0, c, None);
    t.settle();
    assert_eq!(t.zone(capsize), Zone::Graveyard(P0));
}

#[test]
fn nexus_of_becoming_copies_an_x_card_with_x_0() {
    cr!("202.3e", "707.2", "122.6");
    ruling!(
        "Nexus of Becoming",
        "If the exiled card has {X} in its mana cost, X is 0."
    );
    supported("Nexus of Becoming");
    supported("Primordial Hydra");
    // "At the beginning of combat on your turn, draw a card. Then you may exile an
    // artifact or creature card from your hand. If you do, create a token that's a copy of
    // the exiled card, except it's a 3/3 Golem artifact creature in addition to its other
    // types." Primordial Hydra {X}{G}{G}: "This creature enters with X +1/+1 counters on
    // it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nexus of Becoming");
    let hydra = t.hand(P0, "Primordial Hydra");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(hydra)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.zone(hydra), Zone::Exile);
    let token = t.named_on_battlefield("Primordial Hydra");
    assert_eq!(token.len(), 1);
    let token = token[0];
    assert!(t.obj_now(token).is_token());
    // X is 0: mana value 2, and it entered with no +1/+1 counters.
    assert_eq!(t.obj_now(token).chars.mana_value(), 2);
    assert_eq!(t.counters(token, "+1/+1"), 0);
    assert_eq!(t.pt(token), (3, 3));
    assert!(t.obj_now(token).chars.is(CardType::Artifact));
}

#[test]
fn jodah_sees_only_the_front_face_of_a_double_faced_card_in_exile() {
    cr!("712.8a", "709.4b", "202.3");
    ruling!(
        "Jodah, the Unifier",
        "The types and mana value of a double-faced card in exile are determined by the characteristics of its front face. The mana value of a split card is the total mana value of both halves of the split card added together."
    );
    supported("Jodah, the Unifier");
    supported("Sun Quan, Lord of Wu");
    // "Whenever you cast a legendary spell from your hand, exile cards from the top of your
    // library until you exile a legendary nonland card with lesser mana value. You may
    // cast that card without paying its mana cost. Put the rest on the bottom of your
    // library in a random order." Invasion of Segovia (a nonlegendary Battle, {2}{U})
    // transforms into Caetus, a legendary creature: in exile it's not legendary. Fire //
    // Ice has mana value 4 there.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jodah, the Unifier");
    let cards = stack_library(
        &mut t,
        P0,
        &[
            "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
            "Fire // Ice",
            "Isamaru, Hound of Konda",
        ],
    );
    assert_eq!(t.obj_now(cards[1]).chars.mana_value(), 4);
    let sun_quan = t.hand(P0, "Sun Quan, Lord of Wu");
    add_mana(&mut t, P0, ManaType::U, 2);
    add_mana(&mut t, P0, ManaType::C, 4);
    t.answer_yes(P0, true);
    t.cast(P0, sun_quan).go();
    t.resolve_all();
    // Jodah skipped the Invasion and Fire // Ice, and cast Isamaru (mana value 1).
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    assert_eq!(t.named_on_battlefield("Sun Quan, Lord of Wu").len(), 1);
    assert!(matches!(t.zone(cards[0]), Zone::Library(_)));
    assert!(matches!(t.zone(cards[1]), Zone::Library(_)));
}

#[test]
fn blightwing_bandits_card_may_be_cast_for_an_alternative_cost() {
    cr!("118.9", "601.2b", "702.74a", "609.4b");
    ruling!(
        "Blightwing Bandit",
        "You pay the costs for an exiled card if you cast it. You may pay any alternative costs the card has rather than the card's mana cost."
    );
    supported("Blightwing Bandit");
    supported("Mulldrifter");
    // "Whenever you cast your first spell during each opponent's turn, look at the top card
    // of that player's library, then exile it face down. You may play that card for as
    // long as it remains exiled, and mana of any type can be spent to cast it." P0 casts
    // Lightning Bolt during P1's turn: P1's Mulldrifter is exiled face down.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blightwing Bandit");
    let mulldrifter = t.library_top(P1, "Mulldrifter");
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::R, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.zone(mulldrifter), Zone::Exile);
    assert!(t.obj_now(mulldrifter).face_down);
    // In P0's turn, P0 casts it for its evoke cost {2}{U}, with red mana for the {U}.
    t.set_step(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::R, 3);
    let c = t.g.current(mulldrifter);
    let evoke = crate::r_s07_common::cast_methods(&mut t, P0, c)
        .into_iter()
        .find(|m| {
            matches!(
                m,
                mtg_engine::object::CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Evoke)
            )
        })
        .expect("evoke is available");
    let hand = t.hand_size(P0);
    let c = t.g.current(mulldrifter);
    t.cast(P0, c).method(evoke).go();
    t.resolve_all();
    // It entered, drew two cards, and was sacrificed: it's in P1's graveyard.
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.zone(mulldrifter), Zone::Graveyard(P1));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

/// P0 activates Chandra, Torch of Defiance's first ability ("+1: Exile the top card of
/// your library. You may cast that card. If you don't, Chandra deals 2 damage to each
/// opponent.") with Wretched Gryff on top of their library ({7}, emerge {5}{U}: "When
/// you cast this spell, draw a card."). Returns the game, the Gryff, and a Hill Giant P0
/// controls (mana value 4).
fn chandra_exiles_gryff() -> (TestGame, ObjectId, ObjectId) {
    supported("Chandra, Torch of Defiance");
    supported("Wretched Gryff");
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    let gryff = t.library_top(P0, "Wretched Gryff");
    let giant = t.battlefield(P0, "Hill Giant");
    t.activate(P0, chandra, 0, &[]).expect("activate +1");
    (t, gryff, giant)
}

#[test]
fn chandras_exiled_card_may_be_cast_for_its_emerge_cost() {
    cr!("608.2g", "601.2b", "118.9", "702.119a");
    ruling!(
        "Chandra, Torch of Defiance",
        "You pay the costs for the exiled card if you cast it. You may pay alternative costs such as emerge rather than the card's mana cost."
    );
    // P0 casts the Gryff for its emerge cost, sacrificing the Giant: {5}{U} minus 4.
    let (mut t, gryff, giant) = chandra_exiles_gryff();
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.on_battlefield(gryff));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // It was cast: no damage.
    assert_eq!(t.life(P1), 20);
}

#[test]
fn chandras_exiled_card_isnt_cast_without_paying_and_then_she_deals_damage() {
    cr!("608.2g", "601.2h", "608.2c");
    ruling!(
        "Chandra, Torch of Defiance",
        "You pay the costs for the exiled card if you cast it."
    );
    // With no mana, P0 can't pay for the Gryff: it isn't cast, it stays in exile, and
    // Chandra deals 2 damage to each opponent.
    let (mut t, gryff, giant) = chandra_exiles_gryff();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(gryff), Zone::Exile);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn spark_of_creativity_counts_x_as_0_for_the_exiled_cards_mana_value() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Spark of Creativity",
        "If a card in exile has {X} in its mana cost, X is considered to be 0."
    );
    supported("Spark of Creativity");
    // "Choose target creature. Exile the top card of your library. You may have Spark of
    // Creativity deal damage to that creature equal to the exiled card's mana value. If
    // you don't, you may play that card until end of turn." Primordial Hydra ({X}{G}{G})
    // has mana value 2 in exile.
    let mut t = TestGame::new(2);
    let hydra = t.library_top(P0, "Primordial Hydra");
    let giant = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 1);
    let spark = t.hand(P0, "Spark of Creativity");
    t.answer_yes(P0, true);
    t.cast(P0, spark).target(giant).go();
    t.resolve_all();
    assert_eq!(t.zone(hydra), Zone::Exile);
    assert_eq!(t.obj_now(giant).damage, 2);
    // P0 dealt the damage: no permission to play the Hydra.
    add_mana(&mut t, P0, ManaType::G, 2);
    t.g.turn.priority = Some(P0);
    let c = t.g.current(hydra);
    assert!(crate::r_s07_common::cast_methods(&mut t, P0, c).is_empty());
    // Declining the damage, P0 may play it this turn instead.
    let mut t = TestGame::new(2);
    let hydra = t.library_top(P0, "Primordial Hydra");
    let giant = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 1);
    let spark = t.hand(P0, "Spark of Creativity");
    t.answer_yes(P0, false);
    t.cast(P0, spark).target(giant).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0);
    add_mana(&mut t, P0, ManaType::G, 3);
    let c = t.g.current(hydra);
    t.cast(P0, c).x(1).go();
    t.resolve_all();
    assert!(t.on_battlefield(hydra));
    assert_eq!(t.counters(hydra, "+1/+1"), 1);
}

#[test]
fn chaos_wand_casts_an_x_spell_with_x_0() {
    cr!("107.3b", "118.9", "608.2g");
    ruling!(
        "Chaos Wand",
        "If the exiled card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Chaos Wand");
    supported("Blaze");
    // "{4}, {T}: Target opponent exiles cards from the top of their library until they
    // exile an instant or sorcery card. You may cast that card without paying its mana
    // cost. Then put the exiled cards that weren't cast this way on the bottom of that
    // library in a random order." P1 exiles Grizzly Bears, then Blaze ({X}{R}: "Blaze
    // deals X damage to any target.").
    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Chaos Wand");
    let cards = stack_library(&mut t, P1, &["Grizzly Bears", "Blaze"]);
    add_mana(&mut t, P0, ManaType::C, 4);
    t.answer_yes(P0, true);
    t.activate(P0, wand, 0, &[Entity::Player(P1)])
        .expect("activate");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // Blaze was cast with X = 0: no damage. It goes to its owner's graveyard.
    assert_eq!(t.zone(cards[1]), Zone::Graveyard(P1));
    assert_eq!(t.life(P1), 20);
    // The Bears went to the bottom of P1's library.
    assert!(matches!(t.zone(cards[0]), Zone::Library(_)));
    assert_eq!(t.g.player(P1).library.first(), Some(&t.g.current(cards[0])));
}
