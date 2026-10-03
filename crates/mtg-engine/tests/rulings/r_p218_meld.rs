//! Rulings batch P218 — meld (CR 701.42, 712.4) in the Commander variant (CR 903.3b,
//! 903.9).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s13_common::commander_game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_melded_commander_is_the_commander_but_only_the_chosen_card_returns() {
    cr!("903.3b", "903.9a", "701.42a");
    ruling!(
        "Gisela, the Broken Blade",
        "In a Commander game, your commander may be Bruna, the Fading Light or Gisela, the Broken Blade, and the other may be in your deck. If they meld into Brisela, Voice of Nightmares, Brisela will also be your commander; but if Brisela leaves the battlefield, only the card chosen as your commander at the start of the game may be put into the command zone."
    );
    supported("Gisela, the Broken Blade");
    supported("Bruna, the Fading Light");
    let mut t = commander_game();
    let gisela = t.battlefield(P0, "Gisela, the Broken Blade");
    t.g.objects[gisela.0 as usize].is_commander = true;
    t.g.players[0]
        .commander_names
        .push("Gisela, the Broken Blade".into());
    t.battlefield(P0, "Bruna, the Fading Light");
    // Gisela's end step trigger melds them.
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let brisela = t.named_on_battlefield("Brisela, Voice of Nightmares");
    assert_eq!(brisela.len(), 1, "{}", t.dump_log());
    let brisela = brisela[0];
    assert!(t.obj(brisela).is_commander);
    // Brisela dies: P0 puts the commander into the command zone; that's only Gisela.
    t.answer_yes(P0, true);
    destroy(&mut t, brisela);
    t.settle();
    let in_command: Vec<String> = t
        .g
        .command
        .iter()
        .map(|id| t.obj(*id).chars.name.to_string())
        .collect();
    assert_eq!(in_command, vec!["Gisela, the Broken Blade".to_string()]);
    assert!(t.in_graveyard(P0, "Bruna, the Fading Light"));
    let bruna = t.g.find_in_zone(Zone::Graveyard(P0), "Bruna, the Fading Light");
    assert!(!t.obj(bruna[0]).is_commander);
}
