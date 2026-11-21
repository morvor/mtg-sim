//! CR 400.7a: an effect of a resolved spell that changed a permanent spell continues to
//! apply to the permanent it becomes, with the timestamp it got when it was created (CR
//! 613.7b), earlier than the permanent's own (CR 613.7d), even when an "as this enters"
//! replacement effect (CR 614.1c, 614.12a) is applied to the permanent as it enters. So the
//! permanent's own static ability applies after it in the same layer.
//!
//! Painter's Servant ("As this creature enters, choose a color. All cards that aren't on the
//! battlefield, spells, and permanents are the chosen color in addition to their other
//! colors.") turned red on the stack by Chaoslace ("Target spell or permanent becomes
//! red.") is red and the chosen color (CR 613.1e): the Servant's effect adds its color to
//! the red. (Were the Chaoslace effect given a new timestamp as the Servant enters, it would
//! overwrite the Servant's color, leaving it red only.)

use crate::r613_audit_common::{choose_color, color_set, colors};
use crate::r614_audit_common::supported;
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

#[test]
fn painters_servant_turned_red_as_a_spell_is_red_and_its_chosen_color() {
    cr!("400.7a", "613.7", "613.7b", "613.7d", "613.1e", "614.1c", "614.12a");
    ruling!(
        "Painter's Servant",
        "This ability doesn't overwrite any previous colors"
    );
    supported("Painter's Servant");
    supported("Chaoslace");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Painter's Servant");
    let spell = t.cast(P0, card).go();
    t.lands(P0, "Mountain", 1);
    let lace = t.hand(P0, "Chaoslace");
    t.cast(P0, lace).target(spell).go();
    // Chaoslace resolves: the Servant spell is red.
    t.resolve();
    t.g.recompute();
    assert_eq!(colors(&t, spell), color_set(&[Color::Red]));
    // The Servant resolves; blue is chosen as it enters.
    choose_color(&mut t, P0, Color::Blue);
    t.resolve_all();
    let servant = t.g.current(spell);
    assert!(t.on_battlefield(servant));
    t.g.recompute();
    assert_eq!(
        colors(&t, servant),
        color_set(&[Color::Red, Color::Blue]),
        "red from Chaoslace, then blue added by the Servant's own ability"
    );
}
