use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::BufRead;

fn get_vmm_overhead(
    pid: u32,
    guest_memory_size: Option<u32>,
    guest_memory_name: Option<String>,
) -> HashMap<String, u32> {
    let smaps = fs::File::open(format!("/proc/{}/smaps", pid)).unwrap();
    let reader = io::BufReader::new(smaps);

    let mut region_name: String = "".to_string();
    let mut map_size: u32 = 0;
    let mut region_maps = HashMap::new();
    for line in reader.lines() {
        let l = line.unwrap();

        if l.contains('-') {
            // Reset region name and map size
            region_name = "".to_string();
            map_size = 0;

            let values: Vec<&str> = l.split_whitespace().collect();
            // Map name start from column 5
            // 0                1    2        3     4                   5
            // ffffff-fff601000 --xp 00000000 00:00 0                  [vsyscall]
            let name_column = 5;
            if values.len() > name_column {
                region_name = values[name_column].to_string();
            }
        }

        // Each section begins with something that looks like:
        // Size:               2184 kB
        if l.starts_with("Size:") {
            let values: Vec<&str> = l.split_whitespace().collect();
            map_size = values[1].parse::<u32>().unwrap();
        }

        // If this is a map we're taking into account, then we only
        // count the RSS. The sum of all counted RSS is the VMM overhead.
        if l.starts_with("Rss:") {
            let values: Vec<&str> = l.split_whitespace().collect();
            let value = values[1].trim().parse::<u32>().unwrap();

            // We skip the assigned guest RAM map, its RSS is only
            // dependent on the guest actual memory usage.
            // Everything else can be added to the VMM overhead.
            if let Some(name) = &guest_memory_name {
                if region_name.contains(name) {
                    println!("SKIP: {} has size {} kB", region_name, value);
                    continue;
                }
            }
            if let Some(size) = guest_memory_size {
                if map_size == size {
                    println!("SKIP: {} has size {} kB", region_name, value);
                    continue;
                }
            }

            *region_maps.entry(region_name.clone()).or_insert(0) += value;
        }
    }

    region_maps
}

fn parse_args(args: Vec<String>) -> (u32, Option<u32>, Option<String>) {
    let pid = args[1].parse::<u32>().unwrap();
    let mut guest_memory_size: Option<u32> = None;
    let mut guest_memory_name: Option<String> = None;

    if let Some(size) = args[2].strip_prefix("--size=") {
        guest_memory_size = Some(size.parse::<u32>().unwrap());
    } else if let Some(name) = args[2].strip_prefix("--name=") {
        guest_memory_name = Some(name.to_string());
    } else {
        panic!("Parameter 2 should be '--size' or '--name'");
    }

    (pid, guest_memory_size, guest_memory_name)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (pid, guest_memory_size, guest_memory_name) = parse_args(args);
    let regions = get_vmm_overhead(pid, guest_memory_size, guest_memory_name);
    let mut total = 0;

    let mut sorted_regions: Vec<(String, u32)> = regions.into_iter().collect();
    sorted_regions.sort_by_key(|r| r.1);

    for (region_name, value) in sorted_regions.iter() {
        println!("  {:<5}kB -- [{}]", value, region_name);
        total += value;
    }

    println!("Total Overhead: {} kB", total);
}
