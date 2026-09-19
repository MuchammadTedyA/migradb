pub fn strip_table_prefix(table_name: &str) -> &str {
    let prefixes = ["m_", "t_", "sys_", "map_"];
    for p in prefixes {
        if let Some(stripped) = table_name.strip_prefix(p) {
            return stripped;
        }
    }
    table_name
}

pub fn singularize(word: &str) -> String {
    let lower = word.to_lowercase();
    
    // Exact irregular matches
    match lower.as_str() {
        "people" => return "person".to_string(),
        "children" => return "child".to_string(),
        "statuses" => return "status".to_string(),
        "status" => return "status".to_string(),
        "data" => return "data".to_string(),
        "metadata" => return "metadata".to_string(),
        _ => {}
    }

    // Rules
    if lower.ends_with("ies") && lower.len() > 3 {
        // companies -> company, categories -> category
        let prefix = &word[..word.len() - 3];
        return format!("{}y", prefix);
    }

    if lower.ends_with("sses") || lower.ends_with("shes") || lower.ends_with("ches") || lower.ends_with("xes") {
        // addresses -> address, branches -> branch, boxes -> box
        return word[..word.len() - 2].to_string();
    }

    if lower.ends_with('s') && !lower.ends_with("ss") && !lower.ends_with("us") && !lower.ends_with("is") {
        // users -> user, products -> product
        return word[..word.len() - 1].to_string();
    }

    word.to_string()
}

pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

pub fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
    }
}

pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !result.ends_with('_') {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Go-idiomatic field naming respecting common initialisms (ID, URL, API, UUID, IP)
pub fn to_go_field_name(s: &str) -> String {
    let parts: Vec<&str> = s.split(|c| c == '_' || c == '-' || c == ' ')
        .filter(|p| !p.is_empty())
        .collect();

    let mut result = String::new();
    for part in parts {
        let upper = part.to_ascii_uppercase();
        match upper.as_str() {
            "ID" | "UUID" | "URL" | "API" | "IP" | "URI" | "HTTP" | "JSON" | "SQL" => {
                result.push_str(&upper);
            }
            _ => {
                let mut chars = part.chars();
                if let Some(first) = chars.next() {
                    result.push(first.to_ascii_uppercase());
                    result.push_str(chars.as_str());
                }
            }
        }
    }
    result
}

/// Converts raw table name (e.g. `m_companies`, `t_sales_orders`) into clean Entity Name (`Company`, `SalesOrder`)
pub fn clean_entity_name(raw_table_name: &str) -> String {
    let stripped = strip_table_prefix(raw_table_name);
    let parts: Vec<&str> = stripped.split('_').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return to_pascal_case(raw_table_name);
    }

    let mut result = String::new();
    let last_idx = parts.len() - 1;
    for (i, part) in parts.iter().enumerate() {
        if i == last_idx {
            // Singularize only the last word (e.g. sales_orders -> sales + order)
            let singular = singularize(part);
            result.push_str(&to_pascal_case(&singular));
        } else {
            result.push_str(&to_pascal_case(part));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_prefix() {
        assert_eq!(strip_table_prefix("m_companies"), "companies");
        assert_eq!(strip_table_prefix("t_sales_orders"), "sales_orders");
        assert_eq!(strip_table_prefix("sys_users"), "users");
        assert_eq!(strip_table_prefix("map_user_roles"), "user_roles");
        assert_eq!(strip_table_prefix("products"), "products");
    }

    #[test]
    fn test_singularize() {
        assert_eq!(singularize("companies"), "company");
        assert_eq!(singularize("branches"), "branch");
        assert_eq!(singularize("categories"), "category");
        assert_eq!(singularize("users"), "user");
        assert_eq!(singularize("roles"), "role");
        assert_eq!(singularize("statuses"), "status");
    }

    #[test]
    fn test_clean_entity_name() {
        assert_eq!(clean_entity_name("m_companies"), "Company");
        assert_eq!(clean_entity_name("m_branches"), "Branch");
        assert_eq!(clean_entity_name("t_sales_orders"), "SalesOrder");
        assert_eq!(clean_entity_name("sys_users"), "User");
        assert_eq!(clean_entity_name("map_user_roles"), "UserRole");
    }

    #[test]
    fn test_go_field_name() {
        assert_eq!(to_go_field_name("id"), "ID");
        assert_eq!(to_go_field_name("company_id"), "CompanyID");
        assert_eq!(to_go_field_name("user_uuid"), "UserUUID");
        assert_eq!(to_go_field_name("created_at"), "CreatedAt");
    }
}

