pub mod pokemon;
use pokemon::Pokemon;

use dioxus::prelude::*;
use std::fs;
use bincode::{config, decode_from_slice};

#[allow(non_snake_case)]
pub fn App() -> Element {
    let pokedex = use_signal(|| {
        match fs::read("pokedex_default.bin") {
            Ok(bytes) => {
                match decode_from_slice::<Vec<Pokemon>, _>(&bytes, config::standard()) {
                    Ok((data, _)) => data,
                    Err(e) => {
                        eprintln!("Failed to decode: {}", e);
                        Vec::new()
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to read file: {}", e);
                // Print more detailed error information
                eprintln!("Current directory: {:?}", std::env::current_dir().unwrap_or_default());
                Vec::new()
            }
        }
    });

    // Filter states
    let mut name_filter = use_signal(|| "".to_string());
    let mut selected_type1 = use_signal(|| "".to_string());
    let mut selected_type2 = use_signal(|| "".to_string());
    let mut min_hp = use_signal(|| 0);
    let mut min_attack = use_signal(|| 0);
    let mut min_defense = use_signal(|| 0);
    let mut min_sp_atk = use_signal(|| 0);
    let mut min_sp_def = use_signal(|| 0);
    let mut min_speed = use_signal(|| 0);
    let mut selected_gen = use_signal(|| 0); 
    
    // Height and weight filters
    let mut min_height = use_signal(|| 0.0);
    let mut max_height = use_signal(|| 20.0);
    let mut min_weight = use_signal(|| 0.0);
    let mut max_weight = use_signal(|| 1000.0);
    
    // Add these new state variables
    let mut min_gen = use_signal(|| 1);
    let mut max_gen = use_signal(|| 9); // Assuming gen 9 is the latest
    
    // Add this state variable for excluded types
    let mut excluded_types = use_signal(|| Vec::<String>::new());

    // Add these new state variables
    let mut excluded_pokemon = use_signal(|| Vec::<String>::new());
    let mut height_comparison = use_signal(|| "any".to_string()); // "any", "taller", or "shorter"
    let mut weight_comparison = use_signal(|| "any".to_string()); // "any", "heavier", or "lighter"
    let mut reference_pokemon = use_signal(|| None::<Pokemon>);

    // Extract all unique types
    let types = use_memo(move || {
        let mut types = Vec::new();
        for pokemon in pokedex.read().iter() {
            if !types.contains(&pokemon.type1) {
                types.push(pokemon.type1.clone());
            }
            if let Some(type2) = &pokemon.type2 {
                if !type2.is_empty() && !types.contains(type2) {
                    types.push(type2.clone());
                }
            }
        }
        types.sort();
        types
    });
    
    // Extract all generations
    let generations = use_memo(move || {
        let mut gens = Vec::new();
        for pokemon in pokedex.read().iter() {
            if !gens.contains(&pokemon.generation) {
                gens.push(pokemon.generation);
            }
        }
        gens.sort();
        gens
    });

    let max_pokemon_height = use_memo(move || {
        pokedex.read().iter()
            .map(|p| p.height)
            .fold(0.0, f32::max)
            .ceil()
    });
    
    let max_pokemon_weight = use_memo(move || {
        pokedex.read().iter()
            .map(|p| p.weight)
            .fold(0.0, f32::max)
            .ceil()
    });

    // Create a derived state for filtered Pokémon
    let filtered_pokemon = use_memo(move || {
        let filtered = pokedex.read().iter()
            .filter(|p| {
                // Name filter
                let name_match = p.name.to_lowercase().contains(&name_filter().to_lowercase());
                
                // Type filters
                // For Type 1 filtering
                let type1_match = selected_type1().is_empty() || selected_type1() == "All Types" || 
                    p.type1.to_lowercase() == selected_type1().to_lowercase();

                // For Type 2 filtering - completely rewritten with better logging
                let type2_match = {
                    if selected_type2().is_empty() || selected_type2() == "Any/None" {
                        // When "Any/None" is selected, show all Pokémon regardless of Type 2
                        true
                    } else {
                        // When a specific Type 2 is selected, only show Pokémon with that Type 2
                        let has_matching_type2 = p.type2.as_ref()
                            .map(|t| !t.is_empty() && t.to_lowercase() == selected_type2().to_lowercase())
                            .unwrap_or(false);
                        
                        has_matching_type2
                    }
                };
                
                // Generation filter
                let gen_match = p.generation >= min_gen() && p.generation <= max_gen();
            
                // Height and weight filters
                let height_match = p.height >= min_height() && p.height <= max_height();
                let weight_match = p.weight >= min_weight() && p.weight <= max_weight();
                
                // Stat filters
                let hp_match = p.hp >= min_hp();
                let attack_match = p.attack >= min_attack();
                let defense_match = p.defense >= min_defense();
                let sp_atk_match = p.sp_atk >= min_sp_atk();
                let sp_def_match = p.sp_def >= min_sp_def();
                let speed_match = p.speed >= min_speed();
                
                // Excluded types filter
                let not_excluded = {
                    // Check if either type1 or type2 is in the excluded list
                    let type1_not_excluded = !excluded_types().contains(&p.type1);
                    let type2_not_excluded = p.type2.as_ref().map_or(true, |t| 
                        !excluded_types().contains(t)
                    );
                    
                    // A Pokemon is filtered out if either of its types is excluded
                    type1_not_excluded && type2_not_excluded
                };
                
                // Apply all filters
                name_match && type1_match && type2_match && gen_match && 
                height_match && weight_match &&
                hp_match && attack_match && defense_match && 
                sp_atk_match && sp_def_match && speed_match && not_excluded
            })
            .cloned()
            .collect::<Vec<_>>();

        // Add debug diagnostics
        if filtered.is_empty() {
            eprintln!("\n=== FILTER DEBUGGING ===");
            eprintln!("No Pokémon matched the current filters:");
            eprintln!("Type1: '{}', Type2: '{}'", selected_type1(), selected_type2());
            eprintln!("Generation range: {} - {}", min_gen(), max_gen());
            eprintln!("Excluded types: {:?}", excluded_types());
            
            // Count how many pass each individual filter
            let pass_type1 = pokedex.read().iter().filter(|p| {
                selected_type1().is_empty() || selected_type1() == "All Types" || 
                p.type1.to_lowercase() == selected_type1().to_lowercase()
            }).count();
            
            let pass_type2 = pokedex.read().iter().filter(|p| {
                if selected_type2().is_empty() {
                    true
                } else {
                    p.type2.as_ref()
                        .map(|t| !t.is_empty() && t.to_lowercase() == selected_type2().to_lowercase())
                        .unwrap_or(false)
                }
            }).count();
            
            eprintln!("Pokémon passing Type1 filter: {}/{}", pass_type1, pokedex.read().len());
            eprintln!("Pokémon passing Type2 filter: {}/{}", pass_type2, pokedex.read().len());
            
            // Sample a few Pokémon to see why they don't match
            eprintln!("\nSample Pokémon and why they don't match:");
            for (i, p) in pokedex.read().iter().take(5).enumerate() {
                let t1_match = selected_type1().is_empty() || p.type1.to_lowercase() == selected_type1().to_lowercase();
                let t2_match = if selected_type2().is_empty() {
                    true
                } else {
                    p.type2.as_ref()
                        .map(|t| !t.is_empty() && t.to_lowercase() == selected_type2().to_lowercase())
                        .unwrap_or(false)
                };
                
                let gen_range_match = p.generation >= min_gen() && p.generation <= max_gen();
                let type1_excluded = excluded_types().contains(&p.type1);
                let type2_excluded = p.type2.as_ref().map_or(false, |t| excluded_types().contains(t));
                
                eprintln!("#{}: {} (Type1: {}, Type2: {:?}) - Type1 Match: {}, Type2 Match: {}", 
                    i+1, p.name, p.type1, p.type2, t1_match, t2_match);
                eprintln!("  Generation match: {} (pokemon gen: {})", gen_range_match, p.generation);
                eprintln!("  Type exclusions: type1 excluded: {}, type2 excluded: {}", 
                    type1_excluded, type2_excluded);
            }
            eprintln!("=== END DEBUGGING ===\n");
        }

        // For better debugging, print information about the active filters
        eprintln!("=== FILTER STATUS ===");
        eprintln!("Type1: '{}', Type2: '{}'", selected_type1(), selected_type2());
        eprintln!("Pokémon count: {}", filtered.len());

        // If type filters are active, print some examples
        if !selected_type1().is_empty() || !selected_type2().is_empty() {
            eprintln!("\nSample Type Data:");
            for (i, p) in pokedex.read().iter().take(10).enumerate() {
                eprintln!("#{}: {} - Type1: '{}', Type2: '{:?}'", 
                         i+1, p.name, p.type1, p.type2);
            }
        }
        eprintln!("===================");

        filtered
    });

    rsx! {
        div { class: "container",
            h1 { class: "title", "Pokémon Filter App" }
            
            div { class: "filters",
                h2 { "Filters" }
                
                div { class: "filter-row",
                    label { "Name: " }
                    input {
                        r#type: "text",
                        value: "{name_filter}",
                        oninput: move |e| name_filter.set(e.value().clone()),
                        placeholder: "Search by name..."
                    }
                }
                
                div { class: "filter-row",
                    label { "Generation Range: {min_gen} - {max_gen}" }
                    div { class: "range-inputs",
                        input {
                            r#type: "range",
                            min: "1", 
                            max: "9", // Update this based on your actual data
                            value: "{min_gen}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u8>() {
                                    if val <= max_gen() {
                                        min_gen.set(val);
                                    } else {
                                        // If min goes above max, bring max up too
                                        min_gen.set(val);
                                        max_gen.set(val);
                                    }
                                }
                            }
                        }
                        input {
                            r#type: "range",
                            min: "1", 
                            max: "9", // Update this based on your data
                            value: "{max_gen}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u8>() {
                                    if val >= min_gen() {
                                        max_gen.set(val);
                                    } else {
                                        // If max goes below min, bring min down too
                                        max_gen.set(val);
                                        min_gen.set(val);
                                    }
                                }
                            }
                        }
                    }
                }
                
                div { class: "filter-row",
                    label { "Type 1: " }
                    select {
                        value: "{selected_type1}",
                        onchange: move |e| selected_type1.set(e.value().clone()),
                        option { value: "All Types", "All Types" }  // Make value match text
                        {
                            types.read().iter()
                                .filter(|type_name| !excluded_types().contains(type_name))
                                .map(|type_name| {
                                    rsx! {
                                        option { value: "{type_name}", "{type_name}" }
                                    }
                                })
                        }
                    }
                }

                div { class: "filter-row",
                    label { "Type 2: " }
                    select {
                        value: "{selected_type2}",
                        onchange: move |e| selected_type2.set(e.value().clone()),
                        option { value: "Any/None", "Any/None" }  // Make value match text
                        {
                            types.read().iter()
                                .filter(|type_name| !excluded_types().contains(type_name))
                                .map(|type_name| {
                                    rsx! {
                                        option { value: "{type_name}", "{type_name}" }
                                    }
                                })
                        }
                    }
                }
                
                // Add this UI element for the excluded types (after your Type 2 dropdown)
                div { class: "filter-row",
                    label { "Exclude Types: " }
                    div { class: "excluded-types-container",
                        // Show currently excluded types as tags with matching type colors
                        div { class: "excluded-types-tags",
                            {excluded_types().iter().map(|type_name| {
                                let type_name_owned = type_name.clone();
                                rsx! {
                                    div { 
                                        // Apply both classes to get styling and type-specific color
                                        class: "excluded-type-tag {type_name.to_lowercase()}", 
                                        "{type_name}"
                                        button { 
                                            class: "remove-tag",
                                            onclick: move |_| {
                                                let mut current = excluded_types();
                                                current.retain(|t| t != &type_name_owned);
                                                excluded_types.set(current);
                                            },
                                            "×"
                                        }
                                    }
                                }
                            })}
                        }
                        
                        // Dropdown that filters out already-excluded types
                        select {
                            value: "",
                            onchange: move |e| {
                                let value = e.value().clone();
                                if !value.is_empty() && !excluded_types().contains(&value) {
                                    let mut current = excluded_types();
                                    current.push(value);
                                    excluded_types.set(current);
                                }
                            },
                            option { value: "", "Select type to exclude..." }
                            {
                                types.read().iter()
                                    // Filter out types that are already excluded
                                    .filter(|type_name| !excluded_types().contains(type_name))
                                    .map(|type_name| {
                                        rsx! {
                                            option { value: "{type_name}", "{type_name}" }
                                        }
                                    })
                            }
                        }
                    }
                }
                
                div { class: "filter-row",
                    label { "Height Range: {min_height:.1} - {max_height:.1} m" }
                    div { class: "range-inputs",
                        input {
                            r#type: "range",
                            min: "0", 
                            max: "{max_pokemon_height()}",
                            step: "0.1",
                            value: "{min_height}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<f32>() {
                                    if val <= max_height() {
                                        min_height.set(val);
                                    }
                                }
                            }
                        }
                        input {
                            r#type: "range",
                            min: "0", 
                            max: "{max_pokemon_height()}",
                            step: "0.1",
                            value: "{max_height}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<f32>() {
                                    if val >= min_height() {
                                        max_height.set(val);
                                    }
                                }
                            }
                        }
                    }
                }
                
                div { class: "filter-row",
                    label { "Weight Range: {min_weight:.1} - {max_weight:.1} kg" }
                    div { class: "range-inputs",
                        input {
                            r#type: "range",
                            min: "0", 
                            max: "{max_pokemon_weight()}",
                            step: "0.1",
                            value: "{min_weight}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<f32>() {
                                    if val <= max_weight() {
                                        min_weight.set(val);
                                    }
                                }
                            }
                        }
                        input {
                            r#type: "range",
                            min: "0", 
                            max: "{max_pokemon_weight()}",
                            step: "0.1",
                            value: "{max_weight}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<f32>() {
                                    if val >= min_weight() {
                                        max_weight.set(val);
                                    }
                                }
                            }
                        }
                    }
                }
                
                div { class: "filter-row",
                    label { "Min HP: {min_hp}" }
                    input {
                        r#type: "range",
                        min: "0", 
                        max: "255",
                        value: "{min_hp}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u8>() {
                                min_hp.set(val);
                            }
                        }
                    }
                }
                
                div { class: "filter-row",
                    label { "Min Attack: {min_attack}" }
                    input {
                        r#type: "range",
                        min: "0", 
                        max: "255",
                        value: "{min_attack}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u8>() {
                                min_attack.set(val);
                            }
                        }
                    }
                }
                
                div { class: "filter-row",
                    label { "Min Defense: {min_defense}" }
                    input {
                        r#type: "range",
                        min: "0", 
                        max: "255",
                        value: "{min_defense}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u8>() {
                                min_defense.set(val);
                            }
                        }
                    }
                }
                
                div { class: "filter-row",
                    label { "Min Sp. Atk: {min_sp_atk}" }
                    input {
                        r#type: "range",
                        min: "0", 
                        max: "255",
                        value: "{min_sp_atk}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u8>() {
                                min_sp_atk.set(val);
                            }
                        }
                    }
                }
                
                div { class: "filter-row",
                    label { "Min Sp. Def: {min_sp_def}" }
                    input {
                        r#type: "range",
                        min: "0", 
                        max: "255",
                        value: "{min_sp_def}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u8>() {
                                min_sp_def.set(val);
                            }
                        }
                    }
                }
                
                div { class: "filter-row",
                    label { "Min Speed: {min_speed}" }
                    input {
                        r#type: "range",
                        min: "0", 
                        max: "255",
                        value: "{min_speed}",
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u8>() {
                                min_speed.set(val);
                            }
                        }
                    }
                }
                
                button {
                    class: "reset-button",
                    onclick: move |_| {
                        name_filter.set("".to_string());
                        selected_type1.set("All Types".to_string());  // Change to match dropdown
                        selected_type2.set("Any/None".to_string());   // Change to match dropdown
                        selected_gen.set(0);
                        min_hp.set(0);
                        min_attack.set(0);
                        min_defense.set(0);
                        min_sp_atk.set(0);
                        min_sp_def.set(0);
                        min_speed.set(0);
                        min_height.set(0.0);
                        max_height.set(max_pokemon_height());
                        min_weight.set(0.0);
                        max_weight.set(max_pokemon_weight());
                        min_gen.set(1);
                        max_gen.set(9); // Or whatever your max generation is
                        excluded_types.set(Vec::new());
                    },
                    "Reset Filters"
                }
            }

            div { class: "results",
                h2 { "Results" }
                
                table { class: "pokemon-table",
                    thead {
                        tr {
                            th { "Name" }
                            th { "Type" }
                            th { "HP" }
                            th { "Atk" }
                            th { "Def" }
                            th { "Sp.Atk" }
                            th { "Sp.Def" }
                            th { "Speed" }
                            th { "Total" }
                            th { "Height" }
                            th { "Weight" }
                            th { "Gen" }
                        }
                    }
                    tbody {
                        tr {
                            td { colspan: "12", class: "result-count",
                                "Found {filtered_pokemon().len()} Pokémon"
                            }
                        }
                        
                        for pokemon in filtered_pokemon().iter() {
                            PokemonRow { pokemon: pokemon.clone() }
                        }
                    }
                }
            }
        }
    }
}

// Create a separate component for each Pokemon row
#[component]
fn PokemonRow(pokemon: Pokemon) -> Element {
    rsx! {
        tr { key: "{pokemon.name}",
            td { 
                class: "pokemon-name",
                "{pokemon.name}"
                if let Some(form) = &pokemon.form {
                    if !form.is_empty() {
                        span { class: "form", " ({form})" }
                    }
                }
            }
            td { class: "pokemon-type",
                span { class: "type {pokemon.type1.to_lowercase()}", "{pokemon.type1}" }
                if let Some(type2) = &pokemon.type2 {
                    if !type2.is_empty() {
                        span { class: "type {type2.to_lowercase()}", "{type2}" }
                    }
                }
            }
            td { "{pokemon.hp}" }
            td { "{pokemon.attack}" }
            td { "{pokemon.defense}" }
            td { "{pokemon.sp_atk}" }
            td { "{pokemon.sp_def}" }
            td { "{pokemon.speed}" }
            td { class: "total", "{pokemon.total}" }
            td { "{pokemon.height} m" }
            td { "{pokemon.weight} kg" }
            td { "{pokemon.generation}" }
        }
    }
}