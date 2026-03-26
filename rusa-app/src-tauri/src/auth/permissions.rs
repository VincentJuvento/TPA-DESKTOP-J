/// Returns the list of role names that `role_name` inherits permissions from.
/// A role is authorised to perform actions belonging to its own name AND every
/// name returned by this function.
pub fn get_inherited_roles(role_name: &str) -> &'static [&'static str] {
    match role_name {
        // ── Tier 4: Administrator bypasses every role restriction ──────────────
        "the_administrator" => &[
            // Tier 1 staff
            "biologist",
            "chemist",
            "physicist",
            "biological_engineer",
            "agricultural_engineer",
            "mathematician",
            "data_analyst",
            "earth_security_staff",
            "galactic_security_staff",
            "astronaut",
            "aerospace_engineer",
            "civil_engineer",
            "farmer",
            "space_station_settler",
            "psychiatrist_assistant",
            "medical_staff",
            "cleanup_crew",
            "disposal_crew",
            "wastewater_crew",
            "transport_crew",
            "sanitary_inspector",
            // Tier 2
            "earth_security_head",
            "galactic_security_head",
            "settler_commander",
            "psychiatrist",
            "head_of_medicine",
            "head_of_sanitary",
            // Tier 3 directors
            "the_observer",
            "the_artificer",
            "the_statistician",
            "the_guardian",
            "the_anchorman",
            "the_wanderer",
            "the_taskmaster",
            "the_coordinator",
            "the_accountant",
            "the_librarian",
            "the_nomad",
            "the_overseer",
            "the_director",
        ],

        // ── Tier 3: Directors inherit their subsystem subordinates ─────────────
        "the_observer" => &[
            "biologist",
            "chemist",
            "physicist",
            "biological_engineer",
            "agricultural_engineer",
        ],
        "the_artificer" => &["mathematician"],
        "the_statistician" => &["data_analyst"],
        "the_guardian" => &[
            "earth_security_head",
            "earth_security_staff",
            "galactic_security_head",
            "galactic_security_staff",
        ],
        "the_overseer" => &["the_guardian", "the_nomad"],
        "the_wanderer" => &["astronaut", "aerospace_engineer"],
        "the_taskmaster" => &["astronaut", "aerospace_engineer"],
        "the_coordinator" => &["settler_commander", "civil_engineer", "farmer"],
        "the_nomad" => &["settler_commander", "civil_engineer", "farmer"],

        // ── Tier 2: Heads inherit their direct subordinates ────────────────────
        "head_of_sanitary" => &[
            "cleanup_crew",
            "disposal_crew",
            "wastewater_crew",
            "transport_crew",
            "sanitary_inspector",
        ],
        "head_of_medicine" => &["medical_staff", "psychiatrist_assistant"],
        "earth_security_head" => &["earth_security_staff"],
        "galactic_security_head" => &["galactic_security_staff"],
        "settler_commander" => &["civil_engineer", "farmer"],
        "psychiatrist" => &["psychiatrist_assistant"],

        _ => &[],
    }
}

/// Builds the list of all role names this role's permissions extend to
/// (i.e. the subordinate roles it inherits – not including itself).
pub fn build_inherited_permissions(role_name: &str) -> Vec<String> {
    use std::collections::HashSet;

    let mut visited: HashSet<&'static str> = HashSet::new();
    let mut stack: Vec<&'static str> = Vec::new();

    for &r in get_inherited_roles(role_name) {
        stack.push(r);
    }

    while let Some(r) = stack.pop() {
        if !visited.insert(r) {
            continue;
        }
        for &rr in get_inherited_roles(r) {
            stack.push(rr);
        }
    }

    let mut out: Vec<String> = visited.into_iter().map(|r| r.to_string()).collect();
    out.sort();
    out
}

/// Returns `true` if `role_name` is authorised to perform the action that
/// requires `required_role`, either because the roles match exactly or because
/// `required_role` appears in the inherited permissions of `role_name`.
pub fn has_permission(role_name: &str, required_role: &str) -> bool {
    if role_name == "the_administrator" {
        return true;
    }
    if role_name == required_role {
        return true;
    }

    use std::collections::HashSet;

    let mut visited: HashSet<&'static str> = HashSet::new();
    let mut stack: Vec<&'static str> = Vec::new();

    for &r in get_inherited_roles(role_name) {
        if r == required_role {
            return true;
        }
        stack.push(r);
    }

    while let Some(r) = stack.pop() {
        if !visited.insert(r) {
            continue;
        }
        if r == required_role {
            return true;
        }
        for &rr in get_inherited_roles(r) {
            if rr == required_role {
                return true;
            }
            stack.push(rr);
        }
    }

    false
}
