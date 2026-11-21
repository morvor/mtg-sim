# user-decisions-nov11: keyword cost reductions on an alternative cost apply after the increases

Run the usual implementer and reviewer flow (worker.md). The reviewer runs at max.

**The user (Nov 11):** "for thalia + flashback reduction, '... The total cost is the mana cost or alternative cost (as determined in rule 601.2b), plus all additional costs and cost increases, and minus all cost reductions. ...' implies that, since the increased cost is still a flashback cost, it is reduced, and the flashback in the described situation would only be {U}"

**Supporting rule.** CR 118.9d: "If an alternative cost is being paid to cast a spell, any additional costs, cost increases, and cost reductions that affect that spell are applied to that alternative cost. (See rule 601.2f.)"

**Background.** Register entry `keyword-cost-reduction-bounded-by-the-keyword-cost` (docs/interpretations/stack-casting-costs.md; engine-audit-1/casting-costs, db56594). Today the engine applies "[Keyword] costs cost {N} less" to the keyword cost before the other increases, and loses any excess (`kw::modified_keyword_cost`).

**Task.**
1. When the keyword cost is an **alternative cost** (flashback, dash, blitz, and any other alternative cost reduced by an effect that names it), the cost increases that apply to the spell are part of that alternative cost (118.9d). So the keyword reduction comes off the cost after those increases. Examples:
   - Catalyst Stone and Thalia on Deep Analysis's flashback: {1}{U} + {1} − {2} = {U}.
   - Warbringer and Sphere of Resistance on Mardu Scout's dash: {1}{R} + {1} − {2} = {R}.
   - 118.7a still holds: it only reduces generic mana, never below zero.
2. **Additional costs** (buyback, kicker, multikicker, ...) are not covered by the user's reasoning. 118.9d puts increases on an alternative cost, not on an additional one. Keep today's behavior there: Memory Crystal's ruling (2004-10-04) says "Only affects generic mana portions of Buyback costs." and "Does not apply to other parts of the cost." Say this explicitly in the register entry, and flag it in the report so the coordinator can confirm it with the user.
3. Rewrite the register entry:
   - id `keyword-cost-reduction-on-an-alternative-cost-after-increases` (or similar);
   - "Decided by the user, Nov 11";
   - quote the user, 601.2f, 118.7a and 118.9d, and the Memory Crystal, Warbringer and Inquisitive Glimmer rulings;
   - keep the old reading as the rejected alternative;
   - update every `interp!` citing the old id;
   - regenerate docs/INTERPRETATIONS.md.
4. Update the tests that pinned the old reading (tests/cr/r601_audit_cost_order.rs, r601_audit_alternative_additional.rs, tests/keywords/k702_027_buyback.rs), and add the Deep Analysis and Mardu Scout cases above.

Follow worker.md's one-item-per-session rule and notify the coordinator when it lands.
