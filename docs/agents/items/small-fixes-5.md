# small-fixes-5: Omniscience with "flash if you pay more", Kefnet's round trip, the round-trip list

Implementation item (effort high; review at max). Quote every CR rule and ruling you rely on verbatim; check every ruling of the cards involved before calling anything provisional.

1. **Omniscience and a card's own "as though it had flash if you pay {2} more"** (mutation-2/permissions, bug 5; outside permissions.rs, in casting.rs / kw::offered_costs::extend_cast_options): with Omniscience ("You may cast spells from your hand without paying their mana costs."), the card's own flash way is never offered free (Rout, Harbinger of the Tides, Ghitu Fire and the other cards of that wording, 9 in all). CR 601.3c, 118.9d; Omniscience's ruling "You can, however, pay additional costs". Fix generally: an alternative cost (without paying its mana cost) combines with the card's own additional cost that grants flash; tests per wording.
2. **Myr Welder with two imprinted Unstable Amulets** (mutation-2/permissions, bug 4: grants identified only by source object, CR 607.1c, 607.5): first check whether gaps-residual-2 (the per-object ability-link counter, landing separately) fixed it; if not, fix it here; test either way.
3. **God-Eternal Kefnet's "That copy costs {2} less to cast." doesn't round-trip** (small-fixes-4's remaining gap): fix the renderer or the compiled form so it round-trips.
4. **roundtrip-passing.txt**: run the full round-trip check with --write and commit the list (small-fixes-4 found 9 more cards passing from other items: Amphibious Kavu, Dwarven Soldier, Hedron Blade, Lairwatch Giant, Mirror Box, Rashka the Slayer, Serra Inquisitors, Swift Silence, Thijarian Witness); report the before/after count.

`cargo test --workspace` passes; yield row; follow worker.md's one-item-per-session rule and notify the coordinator.
