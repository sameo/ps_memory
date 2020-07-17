use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::BufRead;

fn get_vmm_overhead(pid: u32, guest_memory_size: u32) -> HashMap<String, u32> {
    let smaps = fs::File::open(format!("/proc/{}/smaps", pid)).unwrap();
    let reader = io::BufReader::new(smaps);

    let mut skip_map: bool = false;
    let mut region_name: String = "".to_string();
    let mut region_maps = HashMap::new();
    for line in reader.lines() {
        let l = line.unwrap();

        if l.contains("-") {
            let values: Vec<&str> = l.split_whitespace().collect();
            region_name = values.last().unwrap().trim().to_string();
            if region_name == "0" {
                region_name = "anonymous".to_string()
            }
        }

        // Each section begins with something that looks like:
        // Size:               2184 kB
        if l.starts_with("Size:") {
            let values: Vec<&str> = l.split_whitespace().collect();
            let map_size = values[1].parse::<u32>().unwrap();
            // We skip the assigned guest RAM map, its RSS is only
            // dependent on the guest actual memory usage.
            // Everything else can be added to the VMM overhead.
            skip_map = map_size == guest_memory_size;
            if skip_map {
                println!("SKIP: {} has size {}", region_name, map_size);
            }
            continue;
        }

        // If this is a map we're taking into account, then we only
        // count the RSS. The sum of all counted RSS is the VMM overhead.
        if !skip_map && l.starts_with("Rss:") {
            let values: Vec<&str> = l.split_whitespace().collect();
            let value = values[1].trim().parse::<u32>().unwrap();
            *region_maps.entry(region_name.clone()).or_insert(0) += value;
        }
    }

    region_maps
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let regions = get_vmm_overhead(
        args[1].parse().unwrap(),
        args[2].parse::<u32>().unwrap() * 1024,
    );
    let mut total = 0;

    let mut sorted_regions: Vec<(String, u32)> = regions.into_iter().collect();
    sorted_regions.sort_by_key(|r| r.1);

    for (region_name, value) in sorted_regions.iter() {
        println!("  {:<5}kB -- [{}]", value, region_name);
        total += value;
    }

    println!("Total Overhead: {} kB", total);
}
