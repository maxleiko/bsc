// FIXME change is_XXX to macros

pub fn is_u16(v: String) -> Result<(), String> {
  match v.parse::<u16>() {
    Ok(_) => Ok(()),
    Err(e) => Err(format!("{}", e)),
  }
}

pub fn is_u64(v: String) -> Result<(), String> {
  match v.parse::<u64>() {
    Ok(_) => Ok(()),
    Err(e) => Err(format!("{}", e)),
  }
}

pub fn is_u32(v: String) -> Result<(), String> {
  match v.parse::<u32>() {
    Ok(_) => Ok(()),
    Err(e) => Err(format!("{}", e)),
  }
}