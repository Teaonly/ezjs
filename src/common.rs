pub fn str_to_integer(symbol: &str) -> Option<u64> {
	if let Ok(num) = symbol.parse::<u64>() {
		return Some(num);
	}
	return None;
}

pub fn str_to_number(symbol: &str) -> Option<f64> {
	if let Ok(v) = symbol.parse::<f64>() {
		return Some(v);
	}
	if symbol.starts_with("0x") {
		let symbol:&str = &symbol[2..];
		if let Ok(v) = u64::from_str_radix(&symbol, 16) {
			return Some(v as f64);
		}
	}
	if symbol.starts_with("bx") {
		let symbol:&str = &symbol[2..];
		if let Ok(v) = u64::from_str_radix(&symbol, 2) {
			return Some(v as f64);
		}
	}
	if symbol == "NaN" {
		return Some(f64::NAN);
	}
	if symbol == "Infinity" {
		return Some(f64::INFINITY);
	}
	return None;
}

