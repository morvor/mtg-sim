//! Rulings on tokens whose power and toughness are a value ("create an X/X ... token,
//! where X is ..."): X is determined once, as the effect creates the token, from the game
//! as it is then — using last known information for objects that have left the
//! battlefield (CR 111.3, 608.2h).

use mtg_engine::types::counters;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn flesh_carver_uses_its_power_as_it_died() {
    ruling!(
        "Flesh Carver",
        "Use Flesh Carver’s power when it died (including any +1/+1 counters it had)"
    );
    supported("Flesh Carver");
    let mut t = TestGame::new(2);
    let carver = t.battlefield(P0, "Flesh Carver");
    t.g.add_counters(Entity::Object(carver), counters::PLUS1, 2, None);
    t.g.recompute();
    assert_eq!(t.pt(carver), (4, 4));
    t.g.destroy(carver, None);
    t.resolve_all();
    let horrors = t.named_on_battlefield("Horror Token");
    assert_eq!(horrors.len(), 1);
    assert_eq!(t.pt(horrors[0]), (4, 4));
}

#[test]
fn ooze_garden_uses_the_sacrificed_creatures_last_known_power() {
    ruling!(
        "Ooze Garden",
        "X is equal to the power of the sacrificed creature as it last existed on the battlefield."
    );
    supported("Ooze Garden");
    let mut t = TestGame::new(2);
    let garden = t.battlefield(P0, "Ooze Garden");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 3);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(giant).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (6, 6));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.activate(P0, garden, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    let oozes = t.named_on_battlefield("Ooze Token");
    assert_eq!(oozes.len(), 1);
    assert_eq!(t.pt(oozes[0]), (6, 6));
}

#[test]
fn oviya_counts_creatures_as_the_ability_resolves_not_the_new_token() {
    ruling!(
        "Oviya Pashiri, Sage Lifecrafter",
        "The Construct token you're creating doesn't count towards X"
    );
    ruling!(
        "Oviya Pashiri, Sage Lifecrafter",
        "The token's power and toughness won't change as the number of creatures you control changes."
    );
    supported("Oviya Pashiri, Sage Lifecrafter");
    let mut t = TestGame::new(2);
    let oviya = t.battlefield(P0, "Oviya Pashiri, Sage Lifecrafter");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    t.activate(P0, oviya, 1, &[]).unwrap();
    t.resolve_all();
    let constructs = t.named_on_battlefield("Construct Token");
    assert_eq!(constructs.len(), 1);
    // Oviya and the Bears: 2, not 3.
    assert_eq!(t.pt(constructs[0]), (2, 2));
    t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(constructs[0]), (2, 2));
}

#[test]
fn seed_guardian_counts_itself_in_the_graveyard() {
    ruling!(
        "Seed Guardian",
        "Seed Guardian will count toward this number as long as it’s still in your graveyard"
    );
    ruling!(
        "Seed Guardian",
        "Once the Elemental is created, the number of creature cards in your graveyard changing"
    );
    supported("Seed Guardian");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let guardian = t.battlefield(P0, "Seed Guardian");
    t.g.destroy(guardian, None);
    t.resolve_all();
    let elementals = t.named_on_battlefield("Elemental Token");
    assert_eq!(elementals.len(), 1);
    // Grizzly Bears and Seed Guardian.
    assert_eq!(t.pt(elementals[0]), (2, 2));
    t.graveyard(P0, "Hill Giant");
    assert_eq!(t.pt(elementals[0]), (2, 2));
}

#[test]
fn miming_slime_uses_the_greatest_power_as_it_resolves() {
    ruling!(
        "Miming Slime",
        "It won’t change as the greatest power among creatures you control changes."
    );
    supported("Miming Slime");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    let slime = t.hand(P0, "Miming Slime");
    t.cast(P0, slime).go();
    t.resolve_all();
    let ooze = t.named_on_battlefield("Ooze Token")[0];
    assert_eq!(t.pt(ooze), (3, 3));
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(giant).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (6, 6));
    assert_eq!(t.pt(ooze), (3, 3));
}

#[test]
fn miming_slime_with_no_creatures_makes_a_0_0_token() {
    ruling!(
        "Miming Slime",
        "If you control no creatures at that time, X will be 0, creating a 0/0 Ooze token"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let slime = t.hand(P0, "Miming Slime");
    t.cast(P0, slime).go();
    t.g.resolve_top();
    let ooze = t.named_on_battlefield("Ooze Token")[0];
    assert_eq!(t.pt(ooze), (0, 0));
    t.settle();
    assert!(t.named_on_battlefield("Ooze Token").is_empty());
}

#[test]
fn experimental_overload_doesnt_count_itself() {
    ruling!(
        "Experimental Overload",
        "Because Experimental Overload is still on the stack while it's resolving, it doesn't count towards the value of X."
    );
    ruling!(
        "Experimental Overload",
        "You exile Experimental Overload even if you don't return a card to your hand."
    );
    supported("Experimental Overload");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Divination");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Experimental Overload");
    t.answer_yes(P0, false);
    t.cast(P0, spell).go();
    t.resolve_all();
    let weird = t.named_on_battlefield("Weird Token")[0];
    assert_eq!(t.pt(weird), (2, 2));
    assert!(t.in_graveyard(P0, "Lightning Bolt") && t.in_graveyard(P0, "Divination"));
    assert!(t.in_exile("Experimental Overload"));
}

#[test]
fn devastating_summons_with_zero_lands_makes_two_0_0_tokens() {
    ruling!(
        "Devastating Summons",
        "You may sacrifice zero lands as you cast Devastating Summons."
    );
    supported("Devastating Summons");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Devastating Summons");
    t.cast(P0, spell).x(0).go();
    t.g.resolve_top();
    let tokens = t.named_on_battlefield("Elemental Token");
    assert_eq!(tokens.len(), 2);
    assert!(tokens.iter().all(|o| t.pt(*o) == (0, 0)));
    t.settle();
    assert!(t.named_on_battlefield("Elemental Token").is_empty());
}

#[test]
fn devastating_summons_x_is_the_number_of_lands_sacrificed() {
    cr!("107.3a");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Mountain", 4);
    let spell = t.hand(P0, "Devastating Summons");
    t.answer_choose(P0, &[Entity::Object(lands[2]), Entity::Object(lands[3])]);
    t.cast(P0, spell).x(2).go();
    t.resolve_all();
    let tokens = t.named_on_battlefield("Elemental Token");
    assert_eq!(tokens.len(), 2);
    assert!(tokens.iter().all(|o| t.pt(*o) == (2, 2)));
}

#[test]
fn chainers_torment_token_deals_x_damage_not_its_modified_power() {
    // "Create an X/X ... token, where X is half your life total, rounded up. It deals X
    // damage to you.": the later X is the X the first sentence defined.
    cr!("608.2h");
    ruling!(
        "Chainer's Torment",
        "the amount of damage it deals to you is still X, not its modified power or toughness"
    );
    supported("Chainer's Torment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let saga = t.battlefield(P0, "Chainer's Torment");
    // Chapters I and II first, then chapter III with P0 at 15 life: X is 8 (half of 15,
    // rounded up).
    let lore = t.counters(saga, counters::LORE);
    if lore < 2 {
        t.g.add_counters(Entity::Object(saga), counters::LORE, 2 - lore, None);
        t.resolve_all();
    }
    t.g.players[0].life = 15;
    t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
    t.resolve_all();
    let horror = t.named_on_battlefield("Nightmare Horror Token");
    assert_eq!(horror.len(), 1);
    // The token is 9/9 with Glorious Anthem, but deals 8 damage.
    assert_eq!(t.pt(horror[0]), (9, 9));
    assert_eq!(t.life(P0), 15 - 8);
}
