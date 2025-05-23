use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use csv::{Reader, StringRecord};
use serde::Deserialize;
use pokemon_filter::pokemon::Pokemon;

fn main() -> anyhow::Result<()> {
    println!("Starting Pokemon data conversion...");
    
    // Check if CSV file exists
    let csv_path = "C:/Users/Marshall/Documents/CompletePokemon.csv";
    if !Path::new(csv_path).exists() {
        println!("ERROR: CSV file not found at: {}", csv_path);
        println!("Please make sure the file exists at this path.");
        return Ok(());
    }
    
    println!("Found CSV file: {}", csv_path);
    
    // Create assets directory if it doesn't exist
    let assets_dir = Path::new("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(assets_dir)?;
        println!("Created assets directory");
    }

    // Open the CSV file and read the headers
    let mut rdr = csv::Reader::from_path(csv_path)?;

    // Read the headers to know the column positions
    let headers = rdr.headers()?.clone();
    println!("CSV Headers: {:?}", headers);

    // Find index of columns
    let gen_idx = headers.iter().position(|h| h == "Generation").unwrap_or(0);
    let name_idx = headers.iter().position(|h| h == "Name").unwrap_or(1);
    let form_idx = headers.iter().position(|h| h == "Form").unwrap_or(2);
    let type1_idx = headers.iter().position(|h| h == "Type1").unwrap_or(3);
    let type2_idx = headers.iter().position(|h| h == "Type2").unwrap_or(4);
    let total_idx = headers.iter().position(|h| h == "Total").unwrap_or(5);
    let hp_idx = headers.iter().position(|h| h == "HP").unwrap_or(6);
    let attack_idx = headers.iter().position(|h| h == "Attack").unwrap_or(7);
    let defense_idx = headers.iter().position(|h| h == "Defense").unwrap_or(8);
    let sp_atk_idx = headers.iter().position(|h| h == "Sp. Atk").unwrap_or(9);
    let sp_def_idx = headers.iter().position(|h| h == "Sp. Def").unwrap_or(10);
    let speed_idx = headers.iter().position(|h| h == "Speed").unwrap_or(11);
    let height_idx = headers.iter().position(|h| h == "Height").unwrap_or(12);
    let weight_idx = headers.iter().position(|h| h == "Weight").unwrap_or(13);

    let mut pokemon_count = 0;
    let mut pokemons: Vec<Pokemon> = Vec::new();

    println!("Starting to parse Pokemon data...");
    
    // Parse rows manually
    for result in rdr.records() {
        match result {
            Ok(record) => {
                // Convert record to Pokemon
                let pokemon = match parse_pokemon_from_record(&record, 
                    gen_idx, name_idx, form_idx, type1_idx, type2_idx, 
                    total_idx, hp_idx, attack_idx, defense_idx, 
                    sp_atk_idx, sp_def_idx, speed_idx, height_idx, weight_idx) {
                    Ok(p) => p,
                    Err(e) => {
                        println!("Error parsing row: {}", e);
                        continue;
                    }
                };
                
                pokemon_count += 1;
                if pokemon_count <= 3 {
                    println!("Sample Pokemon {}: {:?}", pokemon_count, pokemon);
                }
                pokemons.push(pokemon);
            }
            Err(e) => {
                println!("Error reading row: {}", e);
            }
        }
    }
    
    println!("Finished parsing. Found {} Pokemon", pokemons.len());
    
    if pokemons.is_empty() {
        println!("ERROR: No Pokemon were loaded from the CSV!");
        println!("Check if the CSV format matches your Pokemon struct.");
        return Ok(());
    }

    // Use bincode v2 API to encode
    let encoded = bincode::encode_to_vec(&pokemons, bincode::config::standard())?;
    println!("Encoded data size: {} bytes", encoded.len());
    
    if encoded.len() < 1000 && pokemons.len() > 10 {
        println!("WARNING: Encoded data seems suspiciously small!");
    }
    
    // Write to both locations
    let main_path = "pokedex_default.bin";
    let mut file = File::create(main_path)?;
    file.write_all(&encoded)?;
    
    let assets_path = "assets/pokedex_default.bin";
    let mut assets_file = File::create(assets_path)?;
    assets_file.write_all(&encoded)?;

    println!("Successfully wrote binary files:");
    println!("  - {}", main_path);
    println!("  - {}", assets_path);
    
    // Verify we can read the data back
    println!("Verifying data can be read back...");
    let bytes = fs::read(main_path)?;
    match bincode::decode_from_slice::<Vec<Pokemon>, _>(&bytes, bincode::config::standard()) {
        Ok((data, _)) => {
            println!("Successfully verified! Read back {} Pokemon", data.len());
            if !data.is_empty() {
                println!("First Pokemon: {:?}", data[0]);
            }
        }
        Err(e) => {
            println!("ERROR: Failed to decode the binary file: {}", e);
        }
    }
    
    Ok(())
}

// Helper function to parse a record into a Pokemon
fn parse_pokemon_from_record(
    record: &StringRecord,
    gen_idx: usize, name_idx: usize, form_idx: usize, type1_idx: usize, type2_idx: usize,
    total_idx: usize, hp_idx: usize, attack_idx: usize, defense_idx: usize,
    sp_atk_idx: usize, sp_def_idx: usize, speed_idx: usize, height_idx: usize, weight_idx: usize
) -> Result<Pokemon, Box<dyn std::error::Error>> {
    let generation = record.get(gen_idx).unwrap_or("0").parse::<u8>()?;
    let name = record.get(name_idx).unwrap_or("").to_string();
    
    let form_str = record.get(form_idx).unwrap_or("").trim();
    let form = if form_str.is_empty() { None } else { Some(form_str.to_string()) };
    
    let type1 = record.get(type1_idx).unwrap_or("").to_string();
    
    let type2_str = record.get(type2_idx).unwrap_or("").trim();
    let type2 = if type2_str.is_empty() { None } else { Some(type2_str.to_string()) };
    
    let total = record.get(total_idx).unwrap_or("0").parse::<u16>()?;
    let hp = record.get(hp_idx).unwrap_or("0").parse::<u8>()?;
    let attack = record.get(attack_idx).unwrap_or("0").parse::<u8>()?;
    let defense = record.get(defense_idx).unwrap_or("0").parse::<u8>()?;
    let sp_atk = record.get(sp_atk_idx).unwrap_or("0").parse::<u8>()?;
    let sp_def = record.get(sp_def_idx).unwrap_or("0").parse::<u8>()?;
    let speed = record.get(speed_idx).unwrap_or("0").parse::<u8>()?;
    let height = record.get(height_idx).unwrap_or("0").parse::<f32>()?;
    let weight = record.get(weight_idx).unwrap_or("0").parse::<f32>()?;
    
    Ok(Pokemon {
        generation, name, form, type1, type2, total, hp, attack, defense,
        sp_atk, sp_def, speed, height, weight
    })
}
