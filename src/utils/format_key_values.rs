use std::collections::HashMap;

pub fn format_key_values(map: &HashMap<String, String>) -> String {
	if map.is_empty() {
		return String::new();
	}

	let max_key_len = map.keys().map(String::len).max().unwrap_or(0);

	let mut sorted_pairs: Vec<_> = map.iter().collect();
	sorted_pairs.sort_by(|a, b| a.0.cmp(b.0));

	sorted_pairs.iter().map(|(key, value)| format!("{key:<max_key_len$} : {value}")).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_when_empty_hashmap_passed() {
		let empty_map: HashMap<String, String> = HashMap::new();
		let result = format_key_values(&empty_map);
		assert_eq!(result, "");
	}

	#[test]
	fn test_when_passed_correct_hashmap() {
		let mut map = HashMap::new();
		map.insert(String::from("Content-Type"), String::from("application/json"));
		map.insert(String::from("Authorization"), String::from("Bearer token123"));
		map.insert(String::from("Host"), String::from("api.example.com"));

		let result = format_key_values(&map);

		let lines: Vec<&str> = result.split('\n').collect();
		assert_eq!(lines.len(), 3);

		assert!(result.contains("Authorization : Bearer token123"));
		assert!(result.contains("Content-Type  : application/json"));
		assert!(result.contains("Host          : api.example.com"));

		// Verify alignment by checking that all colons are at the same position
		let colon_positions: Vec<usize> = lines.iter().map(|line| line.find(" : ").unwrap()).collect();
		// All colon positions should be the same (aligned)
		assert!(colon_positions.windows(2).all(|w| w[0] == w[1]));
	}

	#[test]
	fn test_single_key_value_pair() {
		let mut map = HashMap::new();
		map.insert(String::from("Accept"), String::from("text/html"));

		let result = format_key_values(&map);
		assert_eq!(result, "Accept : text/html");
	}

	#[test]
	fn test_keys_with_different_lengths() {
		let mut map = HashMap::new();
		map.insert(String::from("A"), String::from("short key"));
		map.insert(String::from("Very-Long-Header-Name"), String::from("long key value"));
		map.insert(String::from("Mid"), String::from("medium"));

		let result = format_key_values(&map);

		assert!(result.contains("A                     : short key"));
		assert!(result.contains("Mid                   : medium"));
		assert!(result.contains("Very-Long-Header-Name : long key value"));
	}

	#[test]
	fn test_empty_values() {
		let mut map = HashMap::new();
		map.insert(String::from("Empty-Header"), String::new());
		map.insert(String::from("Normal-Header"), String::from("value"));

		let result = format_key_values(&map);
		assert!(result.contains("Empty-Header  : "));
		assert!(result.contains("Normal-Header : value"));
	}

	#[test]
	fn test_keys_with_special_characters() {
		let mut map = HashMap::new();
		map.insert(String::from("X-Custom-Header"), String::from("custom-value"));
		map.insert(String::from("Content-Length"), String::from("1024"));

		let result = format_key_values(&map);
		assert!(result.contains("Content-Length  : 1024"));
		assert!(result.contains("X-Custom-Header : custom-value"));
	}
}
