//! Rulings batch S14 — revolt: "if a permanent you controlled left the battlefield this
//! turn" (an ability word, CR 207.2c). The condition looks back over the turn at the
//! permanents that left the battlefield under the player's control, whatever the reason
//! and wherever they went; triggered revolt abilities use an intervening "if" clause
//! (CR 603.4).

use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s05_common::{enter, tokens_with_subtype};
use crate::r_s14_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 sacrifices a Treasure token for mana (a cost).
fn sacrifice_treasure(t: &mut TestGame) {
    let treasure = create_token(t, P0, "Treasure");
    t.activate(P0, treasure, 0, &[])
        .expect("Treasure's mana ability");
    t.settle();
    assert!(!t.on_battlefield(treasure));
}

/// P0 casts Airdrop Aeronauts ("Revolt — When this creature enters, if a permanent left
/// the battlefield under your control this turn, you gain 5 life.") and everything
/// resolves. Returns the life P0 gained.
fn aeronauts_gain(t: &mut TestGame) -> i32 {
    let before = t.life(P0);
    cast_from_hand(t, P0, "Airdrop Aeronauts", &[]);
    t.resolve_all();
    t.life(P0) - before
}

#[test]
fn a_token_leaving_the_battlefield_satisfies_revolt() {
    cr!("603.4", "111.7", "701.21a");
    ruling!(
        "Countless Gears Renegade",
        "Tokens that leave the battlefield will satisfy a revolt ability."
    );
    supported("Countless Gears Renegade");
    // Countless Gears Renegade: "Revolt — When this creature enters, if a permanent left
    // the battlefield under your control this turn, create a 1/1 colorless Servo artifact
    // creature token."
    let mut t = TestGame::new(2);
    cast_from_hand(&mut t, P0, "Countless Gears Renegade", &[]);
    t.resolve_all();
    assert!(tokens_with_subtype(&t, P0, "Servo").is_empty());
    sacrifice_treasure(&mut t);
    cast_from_hand(&mut t, P0, "Countless Gears Renegade", &[]);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Servo").len(), 1);
}

#[test]
fn revolt_only_checks_whether_a_permanent_left_not_how_many_or_where_it_is_now() {
    cr!("603.4", "400.7");
    ruling!(
        "Airdrop Aeronauts",
        "Revolt abilities check only whether a permanent you controlled left the battlefield this turn or not. They don't apply multiple times if more than one permanent you controlled left the battlefield. They don't check whether the permanent that left the battlefield is still in the zone it moved to."
    );
    supported("Airdrop Aeronauts");
    supported("Cloudshift");
    // Two Treasures sacrificed: 5 life, once.
    let mut t = TestGame::new(2);
    sacrifice_treasure(&mut t);
    sacrifice_treasure(&mut t);
    assert_eq!(aeronauts_gain(&mut t), 5);
    // Grizzly Bears exiled and returned to the battlefield by Cloudshift: it left.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_from_hand(&mut t, P0, "Cloudshift", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(aeronauts_gain(&mut t), 5);
    // Nothing left: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    assert_eq!(aeronauts_gain(&mut t), 0);
}

#[test]
fn revolt_doesnt_care_why_a_permanent_left_or_where_it_went() {
    cr!("603.4", "701.21a", "701.8a");
    ruling!(
        "Airdrop Aeronauts",
        "Revolt abilities don't care why the permanent left the battlefield, who caused it to move, or where it moved to. They're equally satisfied by an artifact you sacrificed to pay a cost, a creature you controlled that was destroyed by Murder, or an enchantment you returned to your hand with Leave in the Dust."
    );
    supported("Leave in the Dust");
    // An artifact sacrificed to pay a cost.
    let mut t = TestGame::new(2);
    sacrifice_treasure(&mut t);
    assert_eq!(aeronauts_gain(&mut t), 5);
    // A creature destroyed by P1's Murder.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_from_hand(&mut t, P1, "Murder", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(aeronauts_gain(&mut t), 5);
    // An enchantment P0 returns to its hand with Leave in the Dust.
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    cast_from_hand(&mut t, P0, "Leave in the Dust", &[Entity::Object(anthem)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Glorious Anthem"));
    assert_eq!(aeronauts_gain(&mut t), 5);
}

#[test]
fn paying_energy_doesnt_satisfy_revolt() {
    cr!("603.4", "122.1", "107.14");
    ruling!(
        "Airdrop Aeronauts",
        "Energy counters aren't permanents. Paying {E} won't satisfy a revolt ability."
    );
    supported("Aether Hub");
    // Aether Hub: "{T}, Pay {E}: Add one mana of any color."
    let mut t = TestGame::new(2);
    let hub = t.battlefield(P0, "Aether Hub");
    t.g.players[0].counters.insert(counters::ENERGY.into(), 1);
    t.activate(P0, hub, 1, &[])
        .expect("Aether Hub's second ability");
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 0);
    assert_eq!(aeronauts_gain(&mut t), 0);
}

#[test]
fn a_revolt_trigger_needs_a_permanent_to_have_left_already() {
    cr!("603.4");
    ruling!(
        "Airdrop Aeronauts",
        "All cards in the Aether Revolt set with triggered revolt abilities use an intervening \"if\" clause. A permanent you controlled must have left the battlefield earlier in the turn in order for these abilities to trigger; otherwise they do nothing."
    );
    // Airdrop Aeronauts enters with nothing having left: its ability doesn't trigger, and
    // sacrificing a Treasure afterwards doesn't change that.
    let mut t = TestGame::new(2);
    let aeronauts = enter(&mut t, P0, "Airdrop Aeronauts");
    assert_eq!(triggers_from(&t, aeronauts), 0);
    sacrifice_treasure(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn hidden_stockpile_revolt_counts_sacrifices_destruction_and_bounce() {
    cr!("603.4", "701.21a", "701.8a");
    ruling!(
        "Hidden Stockpile",
        "Revolt abilities don't care why the permanent left the battlefield, who caused it to move, or where it moved to. They're equally satisfied by an artifact you sacrificed to pay a cost, a creature you controlled that was destroyed by Cast Down, or an enchantment you returned to your hand with Cyclonic Rift."
    );
    supported("Hidden Stockpile");
    // Hidden Stockpile: "Revolt — At the beginning of your end step, if a permanent left
    // the battlefield under your control this turn, create a 1/1 colorless Servo artifact
    // creature token. {1}, Sacrifice a creature: Scry 1."
    for how in ["sacrificed", "destroyed", "bounced", "nothing"] {
        let mut t = TestGame::new(2);
        let stockpile = t.battlefield(P0, "Hidden Stockpile");
        let bears = t.battlefield(P0, "Grizzly Bears");
        match how {
            "sacrificed" => {
                t.lands(P0, "Wastes", 1);
                t.answer_choose(P0, &[Entity::Object(bears)]);
                t.activate(P0, stockpile, 0, &[])
                    .expect("sacrifice a creature");
                t.resolve_all();
            }
            "destroyed" => {
                cast_from_hand(&mut t, P1, "Cast Down", &[Entity::Object(bears)]);
                t.resolve_all();
            }
            "bounced" => {
                cast_from_hand(&mut t, P1, "Unsummon", &[Entity::Object(bears)]);
                t.resolve_all();
            }
            _ => {}
        }
        assert_eq!(t.on_battlefield(bears), how == "nothing", "{how}");
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let servos = tokens_with_subtype(&t, P0, "Servo").len();
        assert_eq!(servos, usize::from(how != "nothing"), "{how}");
    }
}
