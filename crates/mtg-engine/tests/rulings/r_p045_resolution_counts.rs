//! Rulings batch P045 — values an ability counts (the creatures in your party, devotion,
//! Allies) are counted as the ability resolves (CR 608.2h), and what the triggered
//! abilities of discard creatures do when their source has left the battlefield.

use crate::r_p045_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `name` enters for P0 (its "enters" trigger on the stack, targeting P1 or P1's
/// Grizzly Bears); if `bounce`, P0 returns it to hand with Unsummon in response. Then
/// everything resolves. P1 holds two Grizzly Bears and controls one.
fn party_creature_enters(name: &str, bounce: bool) -> (TestGame, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    bears_in_hand(&mut t, P1, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let id = t.enter(P0, name);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{name}");
    // (Drop the target answers the ability didn't use.)
    t.clear_answers();
    if bounce {
        let u = t.hand(P0, "Unsummon");
        t.lands(P0, "Island", 1);
        t.cast(P0, u).target(id).go();
        t.resolve();
        assert_eq!(t.zone(id), Zone::Hand(P0));
    }
    t.resolve_all();
    (t, bears)
}

#[test]
fn the_party_is_counted_as_the_ability_resolves() {
    cr!("608.2h", "700.8", "700.8a");
    ruling!(
        "Malakir Blood-Priest",
        "If an ability of a creature counts the number of creatures in your party, that number is counted as the ability resolves."
    );
    ruling!(
        "Acquisitions Expert",
        "If an ability of a creature counts the number of creatures in your party, that number is counted as the ability resolves."
    );
    ruling!(
        "Cascade Seer",
        "If an ability of a creature counts the number of creatures in your party, that number is counted as the ability resolves."
    );
    ruling!(
        "Drana's Silencer",
        "If an ability of a creature counts the number of creatures in your party, that number is counted as the ability resolves."
    );
    ruling!(
        "Squad Commander",
        "If an ability of a creature counts the number of creatures in your party, that number is counted as the ability resolves."
    );
    ruling!(
        "Acquisitions Expert",
        "If there are no creatures in your party as Acquisitions Expert's ability resolves, the target opponent reveals no cards and you choose no card."
    );
    // Each creature is the only member of its party: on the battlefield as the ability
    // resolves, the party is 1; returned to hand in response, it's 0.
    for bounce in [false, true] {
        let x = if bounce { 0 } else { 1 };
        // Malakir Blood-Priest (Cleric): each opponent loses X life, you gain X.
        let (t, _) = party_creature_enters("Malakir Blood-Priest", bounce);
        assert_eq!((t.life(P1), t.life(P0)), (20 - x, 20 + x));
        // Acquisitions Expert (Rogue): reveal X cards, you choose one, it's discarded.
        let (t, _) = party_creature_enters("Acquisitions Expert", bounce);
        assert_eq!(t.hand_size(P1), 2 - x as usize);
        // Squad Commander (Warrior): a Kor Warrior token for each creature in the party.
        let (t, _) = party_creature_enters("Squad Commander", bounce);
        assert_eq!(tokens(&t, P0).len(), x as usize);
        // Drana's Silencer (Rogue): target creature an opponent controls gets -X/-X.
        let (t, bears) = party_creature_enters("Drana's Silencer", bounce);
        assert_eq!(t.pt(bears), (2 - x, 2 - x));
        // Cascade Seer (Wizard): scry X.
        let (t, _) = party_creature_enters("Cascade Seer", bounce);
        let scries = t
            .asked()
            .into_iter()
            .filter(|(_, d)| matches!(d, Decision::Scry { .. }))
            .count();
        assert_eq!(scries, x as usize);
    }
}

#[test]
fn a_party_member_that_joins_before_resolution_counts() {
    cr!("608.2h", "700.8");
    // Squad Commander (a Warrior) enters; a Cleric joins the party before its ability
    // resolves: two tokens.
    let mut t = TestGame::new(2);
    t.enter(P0, "Squad Commander");
    t.settle();
    t.battlefield(P0, "Malakir Blood-Priest");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn disciple_of_phenax_counts_devotion_on_resolution_including_itself() {
    cr!("608.2h", "700.5");
    ruling!(
        "Disciple of Phenax",
        "Use your devotion to black as the triggered ability resolves to determine how many cards will be revealed. If Disciple of Phenax is still on the battlefield at that time, its mana cost will count toward your devotion to black."
    );
    // On the battlefield: devotion 2 ({2}{B}{B}): two of P1's cards are revealed and P0
    // chooses one of them.
    let (t, _) = party_creature_enters("Disciple of Phenax", false);
    assert_eq!(t.hand_size(P1), 1);
    let revealed: Vec<usize> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates.len()),
            _ => None,
        })
        .collect();
    assert!(revealed.contains(&2), "{revealed:?}");
    // Returned to hand in response: devotion 0, nothing is revealed or discarded.
    let (t, _) = party_creature_enters("Disciple of Phenax", true);
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn bala_ged_thief_with_more_allies_than_cards_reveals_the_whole_hand() {
    cr!("608.2h", "701.20a");
    ruling!(
        "Bala Ged Thief",
        "If the number of Allies you control as the ability resolves is greater than the number of cards in the targeted player’s hand, that player reveals their entire hand."
    );
    supported("Bala Ged Thief");
    // A Bala Ged Thief is on the battlefield; a second one enters: both trigger. P1 has
    // one card: each resolution reveals the whole hand; the first discards the card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bala Ged Thief");
    bears_in_hand(&mut t, P1, 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Bala Ged Thief");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn kaitos_pursuit_menace_is_locked_in_on_resolution() {
    cr!("611.2c", "702.111a");
    ruling!(
        "Kaito's Pursuit",
        "The Ninjas and Rogues that gain menace are determined only once, as Kaito's Pursuit resolves. A Ninja or Rogue that enters the battlefield after it resolves will not have menace"
    );
    supported("Kaito's Pursuit");
    supported("Amoeboid Changeling");
    let mut t = TestGame::new(2);
    // Nezumi Informant is a Rat Rogue.
    let rogue = t.battlefield(P0, "Nezumi Informant");
    let changeling = t.battlefield(P0, "Amoeboid Changeling");
    let k = t.hand(P0, "Kaito's Pursuit");
    give_mana_for(&mut t, P0, "Kaito's Pursuit");
    t.cast(P0, k).target(Entity::Player(P1)).go();
    t.resolve_all();
    let menace = |t: &TestGame, id| t.obj_now(id).has_keyword(KeywordKind::Menace);
    assert!(menace(&t, rogue));
    let later = t.battlefield(P0, "Nezumi Informant");
    assert!(!menace(&t, later));
    // Amoeboid Changeling: "{T}: Target creature loses all creature types until end of
    // turn." The Rogue that stops being a Rogue keeps menace.
    t.activate(P0, changeling, 1, &[Entity::Object(rogue)])
        .unwrap();
    t.resolve_all();
    assert!(!t.obj_now(rogue).chars.has_subtype("Rogue"));
    assert!(menace(&t, rogue));
}

#[test]
fn okiba_gang_shinobi_discards_two_or_fewer_cards() {
    cr!("510.3a", "101.3");
    ruling!(
        "Okiba-Gang Shinobi",
        "If the defending player has two or fewer cards in hand when Okiba-Gang Shinobi deals combat damage to that player, they are all discarded."
    );
    supported("Okiba-Gang Shinobi");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Okiba-Gang Shinobi");
    bears_in_hand(&mut t, P1, 2);
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    t.advance_to(P0, mtg_engine::turn::Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn mesmeric_fiend_leaving_early_exiles_the_card_indefinitely() {
    cr!("603.6c", "610.3c", "607.2a");
    ruling!(
        "Mesmeric Fiend",
        "If Mesmeric Fiend leaves the battlefield before its first ability has resolved, its second ability will trigger and do nothing. Then its first ability will resolve and exile a nonland card from the target opponent's hand indefinitely."
    );
    supported("Mesmeric Fiend");
    let mut t = TestGame::new(2);
    bears_in_hand(&mut t, P1, 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let fiend = t.enter(P0, "Mesmeric Fiend");
    t.settle();
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, shock).target(fiend).go();
    t.resolve(); // Shock: the Fiend dies; its leaves trigger goes on the stack.
    assert!(t.in_graveyard(P0, "Mesmeric Fiend"));
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn kyoki_exiles_face_up_and_the_cards_stay_exiled() {
    cr!("406.3", "610.3");
    ruling!(
        "Kyoki, Sanity's Eclipse",
        "Cards exiled by Kyoki’s ability are exiled face up."
    );
    ruling!(
        "Kyoki, Sanity's Eclipse",
        "Cards exiled by Kyoki’s ability stay exiled even if Kyoki leaves the battlefield."
    );
    supported("Kyoki, Sanity's Eclipse");
    let mut t = TestGame::new(2);
    let kyoki = t.battlefield(P0, "Kyoki, Sanity's Eclipse");
    let bears = bears_in_hand(&mut t, P1, 1)[0];
    // Kami of Ancient Law is a Spirit.
    let kami = t.hand(P0, "Kami of Ancient Law");
    give_mana_for(&mut t, P0, "Kami of Ancient Law");
    t.cast(P0, kami).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!t.obj_now(bears).face_down);
    t.g.destroy(kyoki, None);
    t.resolve_all();
    assert!(!t.on_battlefield(kyoki));
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn parallax_nexus_target_opponent_chooses_the_card() {
    cr!("701.9a", "608.2d");
    ruling!(
        "Parallax Nexus",
        "The target opponent chooses which card to exile."
    );
    supported("Parallax Nexus");
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P0, "Parallax Nexus");
    t.g.objects[nexus.0 as usize]
        .counters
        .insert(counters::FADE.into(), 5);
    let cards = give_hand(&mut t, P1, &["Grizzly Bears", "Hill Giant"]);
    t.answer_choose(P1, &[Entity::Object(cards[1])]);
    let from = t.asked().len();
    t.activate(P0, nexus, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(t.zone(cards[1]), Zone::Exile);
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn nimble_larcenist_must_choose_a_card_if_one_qualifies() {
    cr!("701.20a", "608.2d");
    ruling!(
        "Nimble Larcenist",
        "You must choose a card for the target player to exile if at least one card revealed this way is an artifact, instant, or sorcery card."
    );
    supported("Nimble Larcenist");
    let mut t = TestGame::new(2);
    give_hand(&mut t, P1, &["Grizzly Bears", "Lightning Bolt"]);
    // P0 tries to choose nothing: a card is still exiled.
    t.answer_choose(P0, &[]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Nimble Larcenist");
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn pick_the_brain_with_delirium_searches_and_shuffles_even_with_no_choice() {
    cr!("701.23a", "701.24a", "608.2c");
    ruling!(
        "Pick the Brain",
        "If there is no nonland card to choose, and you have four or more card types among cards in your graveyard, you'll search that player's library even though you won't be able to find any cards. That library will be shuffled."
    );
    supported("Pick the Brain");
    let mut t = TestGame::new(2);
    // Delirium: land, creature, instant, sorcery in P0's graveyard.
    for c in ["Island", "Grizzly Bears", "Lightning Bolt", "Mind Rot"] {
        t.graveyard(P0, c);
    }
    give_hand(&mut t, P1, &["Island"]);
    let p = t.hand(P0, "Pick the Brain");
    give_mana_for(&mut t, P0, "Pick the Brain");
    t.cast(P0, p).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Island"));
    let shuffled = t
        .g
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == P1))
        .count();
    assert_eq!(shuffled, 1);
}
