use conjure_core::rule::Rule;
use linkme::distributed_slice;

#[distributed_slice]
pub static _RULES_DISTRIBUTED_SLICE: [Rule<'static>];

pub fn get_rules() -> Vec<Rule<'static>> {
    _RULES_DISTRIBUTED_SLICE.to_vec()
}

mod rules;

static _TEMP: &Rule = &rules::CONJURE_GEN_RULE_IDENTITY; // Temporary hack to force the static to be included in the binary
